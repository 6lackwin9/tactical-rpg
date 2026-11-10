//! WEBREL1-E1a runtime acceptance (`docs/specs/WEBREL1.md` 6.1, `FB-147`): runtime code reads the wall clock only through
//! `aegis_core::clock`. std's `Instant::now` / `SystemTime::now` compile on `wasm32-unknown-unknown` and panic there at
//! run time, so a clean wasm `cargo check` proves nothing; this lint refuses `std::time::{Instant, SystemTime,
//! UNIX_EPOCH}` (also through `std::time::*`, `use std::time` or a nested `use std::{time::..}`) and `web_time` in the
//! non-test, non-CLI `src` of every engine, kit and game crate.
//!
//! What is not scanned (test-only and native-only items, `src/bin/`, comments, literals): `lint/mod.rs`. CLI crates:
//! `CLI_CRATES`. Allowed sites, each with its reason: `ALLOWED`. The file system twin is `fs_lint.rs`.

mod lint;

use lint::{child_dir, lint_tree, root, src_dirs, Rule};
use std::path::Path;

/// Files that may name the std clock types, each with its reason (keep it short).
const ALLOWED: &[(&str, &str)] = &[
    ("engine/crates/aegis_core/src/clock.rs", "the clock seam itself"),
];

/// Crates that are command-line tools only (never part of the web build).
const CLI_CRATES: &[&str] = &["aegis_cli"];

/// Names under `std::time` that read or hold the wall clock (`Duration` is plain arithmetic and works on the web).
const CLOCK_NAMES: &[&str] = &["Instant", "SystemTime", "UNIX_EPOCH"];

/// Why `p` (a full path, `*` for a glob) is refused, if it is.
fn refused(p: &[&str]) -> Option<String> {
    match p {
        ["web_time", ..] => Some("names web_time (read the clock through aegis_core::clock)".into()),
        ["std", "time"] | ["std", "time", "self" | "*", ..] => Some(format!("imports {} (use aegis_core::clock)", p.join("::"))),
        ["std", "time", name, ..] if CLOCK_NAMES.contains(name) => Some(format!("names std::time::{name} (use aegis_core::clock::{name})")),
        _ => None,
    }
}

const RULE: Rule = Rule { heads: &["std", "web_time"], refused, probes: &[] };

fn violations(text: &str) -> (Vec<(usize, String)>, Vec<String>) {
    lint::violations(text, &RULE)
}

#[test]
fn runtime_code_reads_the_clock_only_through_aegis_core_clock() {
    let root = root();
    for (p, _) in ALLOWED {
        assert!(root.join(p).is_file(), "an allowed site that no longer exists: {p} (drop it from ALLOWED)");
    }
    let found = lint_tree(&root, &RULE, ALLOWED, CLI_CRATES);
    assert!(found.is_empty(), "std::time clock types outside aegis_core::clock (WEBREL1-E1a; std's clock panics on the web):\n{}", found.join("\n"));
}

fn lines_of(src: &str) -> Vec<usize> {
    violations(src).0.into_iter().map(|(l, _)| l).collect()
}

#[test]
fn the_lint_catches_every_spelling_of_the_std_clock() {
    let cases = [
        "use std::time::Instant;",
        "use std::time::{Duration, Instant};",
        "use std::time::{\n    Duration,\n    SystemTime,\n};",
        "fn f() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }",
        "use std::time::*;",
        "use std::time;",
        "use std::time as t;",
        "use std::time::{self, Duration};",
        "use std::{fs, time::Instant};",
        "use std::{fs, time::{Duration, UNIX_EPOCH}};",
        "use ::std::time::Instant;",
        "use web_time::Instant;",
        "struct S { at: Option<std::time::Instant> }",
    ];
    for src in cases {
        assert!(!lines_of(src).is_empty(), "missed: {src}");
    }
    assert_eq!(lines_of("fn f() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }").len(), 2, "both paths of one line are found");
    assert_eq!(lines_of("use std::fs;\n\nuse std::time::{Duration,\n Instant};"), vec![4], "the line of the clock name");
}

#[test]
fn the_lint_passes_the_clock_seam_duration_comments_strings_and_test_code() {
    let clean = [
        "use std::time::Duration;",
        "use aegis_core::clock::{Duration, Instant};",
        "fn f() { let t = aegis_core::clock::Instant::now(); }",
        "// std::time::Instant is the panic on the web\nfn f() {}",
        "/* use std::time::Instant; /* nested */ still a comment */ fn f() {}",
        "const S: &str = \"std::time::Instant\";",
        "const R: &str = r#\"use std::time::SystemTime; \"quoted\"\"#;",
        "fn f<'a>(s: &'a str) -> char { let _ = '\"'; let _ = '{'; 'x' }",
        "#[cfg(test)]\nmod tests {\n    use std::time::Instant;\n    fn f() { let _ = Instant::now(); }\n}",
        "#[cfg(test)]\nuse std::time::Instant;",
        "#[cfg(test)]\n#[allow(dead_code)]\nfn helper() { let _ = std::time::Instant::now(); }",
        "use crate::std::time::Instant;",
    ];
    for src in clean {
        assert!(lines_of(src).is_empty(), "flagged: {src}: {:?}", violations(src).0);
    }
    // the skip of a test item ends with it: runtime code after a test module is still scanned
    assert_eq!(lines_of("#[cfg(test)]\nmod tests {\n    fn f() { if true { } }\n}\nuse std::time::Instant;"), vec![5]);
    // a test-only field, variant or arm ends at its `,` or the enclosing close, not at the item's end
    assert_eq!(lines_of("struct S {\n    #[cfg(test)]\n    a: u8,\n    b: std::time::Instant,\n}"), vec![4]);
    assert_eq!(lines_of("struct S {\n    #[cfg(test)]\n    a: u8\n}\nuse std::time::Instant;"), vec![5]);
    assert_eq!(lines_of("enum E {\n    #[cfg(test)]\n    A(u8, u8),\n    B(std::time::Instant),\n}"), vec![4]);
    assert!(lines_of("#[cfg(test)]\nfn f(a: u8, b: Result<u8, u8>) -> Vec<u8> { let _ = std::time::Instant::now(); vec![a] }").is_empty());
    // and a char literal holding a brace does not end the skip early
    assert_eq!(lines_of("#[cfg(test)]\nmod tests {\n    const C: char = '}';\n    use std::time::Instant;\n}\nfn f() {}"), Vec::<usize>::new());
}

#[test]
fn an_out_of_line_test_module_and_the_excluded_folders_are_skipped() {
    let (_, mods) = violations("mod board;\n#[cfg(test)]\nmod tests;\n");
    assert_eq!(mods, vec!["tests".to_string()]);
    assert_eq!(child_dir(Path::new("src/battle.rs")), Path::new("src/battle"));
    assert_eq!(child_dir(Path::new("src/lib.rs")), Path::new("src"));
    let root = root();
    let dirs = src_dirs(&root, CLI_CRATES);
    assert!(dirs.iter().any(|d| d.ends_with("aegis_ui/src")), "engine crates are walked");
    assert!(dirs.iter().any(|d| d.starts_with(root.join("kits"))), "kit crates are walked");
    assert!(dirs.iter().any(|d| d.starts_with(root.join("games"))), "game crates are walked");
    assert!(!dirs.iter().any(|d| CLI_CRATES.iter().any(|c| d.ends_with(format!("{c}/src")))), "CLI crates are not walked");
}
