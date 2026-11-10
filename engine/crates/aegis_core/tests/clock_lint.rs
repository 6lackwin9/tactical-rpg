//! WEBREL1-E1a runtime acceptance (`docs/specs/WEBREL1.md` 6.1, `FB-147`): runtime code reads the wall clock only through
//! `aegis_core::clock`. std's `Instant::now` / `SystemTime::now` compile on `wasm32-unknown-unknown` and panic there at
//! run time, so a clean wasm `cargo check` proves nothing; this lint refuses `std::time::{Instant, SystemTime,
//! UNIX_EPOCH}` (also through `std::time::*`, `use std::time` or a nested `use std::{time::..}`) and `web_time` in the
//! non-test, non-CLI `src` of every engine, kit and game crate.
//!
//! Not scanned: `tests/`, `benches/`, `examples/` (only `src/` is walked), `src/bin/` (CLI and dev tools), the CLI
//! crates in `CLI_CRATES`, `#[cfg(test)]` items (and the files of a `#[cfg(test)] mod name;`), comments and string or
//! char literals. Allowed sites, each with its reason: `ALLOWED`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Files that may name the std clock types, each with its reason (keep it short).
const ALLOWED: &[(&str, &str)] = &[
    ("engine/crates/aegis_core/src/clock.rs", "the clock seam itself"),
    ("engine/crates/aegis_ui/src/tuning.rs", "a tuning file's modified time from std::fs (not a clock read; the hot reload is a native file read, the read seam is WEBREL1-E1b)"),
];

/// Crates that are command-line tools only (never part of the web build).
const CLI_CRATES: &[&str] = &["aegis_cli"];

/// Names under `std::time` that read or hold the wall clock (`Duration` is plain arithmetic and works on the web).
const CLOCK_NAMES: &[&str] = &["Instant", "SystemTime", "UNIX_EPOCH"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().expect("workspace root")
}

/// True when the `r` at `c[i]` can open a raw string: no identifier before it, or a lone `b` (`br".."`).
fn starts_literal(c: &[char], i: usize) -> bool {
    let ident = |k: usize| c[k].is_alphanumeric() || c[k] == '_';
    i == 0 || !ident(i - 1) || (c[i - 1] == 'b' && (i == 1 || !ident(i - 2)))
}

/// `text` with comments and the insides of string and char literals blanked (newlines kept, so line numbers hold).
fn strip(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let blank = |ch: char| if ch == '\n' { '\n' } else { ' ' };
    let mut i = 0;
    while i < c.len() {
        let next = c.get(i + 1).copied();
        if c[i] == '/' && next == Some('/') {
            while i < c.len() && c[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if c[i] == '/' && next == Some('*') {
            let mut depth = 0;
            loop {
                if i >= c.len() {
                    break;
                }
                if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(blank(c[i]));
                    i += 1;
                }
            }
        } else if c[i] == 'r' && (next == Some('"') || next == Some('#')) && starts_literal(&c, i) {
            // a raw string r"..", r#".."#: ends at the quote followed by as many hashes
            let mut j = i + 1;
            let mut hashes = 0;
            while c.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if c.get(j) != Some(&'"') {
                out.push(c[i]);
                i += 1;
                continue;
            }
            out.extend(std::iter::repeat(' ').take(j + 1 - i));
            i = j + 1;
            while i < c.len() {
                if c[i] == '"' && (1..=hashes).all(|k| c.get(i + k) == Some(&'#')) {
                    out.extend(std::iter::repeat(' ').take(hashes + 1));
                    i += hashes + 1;
                    break;
                }
                out.push(blank(c[i]));
                i += 1;
            }
        } else if c[i] == '"' {
            out.push(' ');
            i += 1;
            while i < c.len() && c[i] != '"' {
                if c[i] == '\\' {
                    out.push(' ');
                    i += 1;
                }
                if i < c.len() {
                    out.push(blank(c[i]));
                    i += 1;
                }
            }
            out.push(' ');
            i += 1;
        } else if c[i] == '\'' && (next == Some('\\') || c.get(i + 2) == Some(&'\'')) {
            // a char literal ('x', '\n', '\u{..}'); a lifetime ('a) has no closing quote two chars on
            out.push(' ');
            i += 1;
            while i < c.len() && c[i] != '\'' {
                if c[i] == '\\' {
                    out.push(' ');
                    i += 1;
                }
                if i < c.len() {
                    out.push(blank(c[i]));
                    i += 1;
                }
            }
            out.push(' ');
            i += 1;
        } else {
            out.push(c[i]);
            i += 1;
        }
    }
    out
}

/// Blanks every `#[cfg(test)]` item of stripped `text` (to its `;` or its closing brace, or, for a field, variant or
/// match arm, to its `,` or the enclosing close it did not open); returns the names of the `#[cfg(test)] mod name;`
/// it met (their files are test-only too).
fn drop_test_items(text: &mut String) -> Vec<String> {
    const ATTR: &str = "#[cfg(test)]";
    let mut mods = Vec::new();
    while let Some(start) = text.find(ATTR) {
        let b = text.as_bytes();
        let mut i = start + ATTR.len();
        let mut depth = 0usize;
        // `(`, `[` and generic `<` nesting outside braces, so a `,` in a signature does not end the item
        let mut nest = 0usize;
        while i < b.len() {
            match b[i] {
                b'{' => depth += 1,
                b'}' if depth == 0 => break,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                b';' if depth == 0 => {
                    i += 1;
                    break;
                }
                b'(' | b'[' | b'<' if depth == 0 => nest += 1,
                b'>' if depth == 0 && i > 0 && (b[i - 1] == b'-' || b[i - 1] == b'=') => {}
                b')' | b']' | b'>' if depth == 0 => nest = nest.saturating_sub(1),
                b',' if depth == 0 && nest == 0 => {
                    i += 1;
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        let item = &text[start + ATTR.len()..i.min(b.len())];
        let words: Vec<&str> = item.split(|ch: char| !(ch.is_alphanumeric() || ch == '_')).filter(|w| !w.is_empty()).collect();
        if item.trim_end().ends_with(';') && !item.contains('{') {
            if let Some(p) = words.iter().position(|w| *w == "mod") {
                if let Some(name) = words.get(p + 1) {
                    mods.push(name.to_string());
                }
            }
        }
        let blanked: String = text[start..i.min(b.len())].chars().map(|ch| if ch == '\n' { '\n' } else { ' ' }).collect();
        text.replace_range(start..i.min(text.len()), &blanked);
    }
    mods
}

#[derive(Debug, PartialEq)]
enum Tok {
    Ident(String, usize),
    Path,
    Punct(char),
}

fn tokens(text: &str) -> Vec<Tok> {
    let c: Vec<char> = text.chars().collect();
    let (mut out, mut line, mut i) = (Vec::new(), 1, 0);
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if ch.is_alphanumeric() || ch == '_' {
            let s = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(Tok::Ident(c[s..i].iter().collect(), line));
        } else if ch == ':' && c.get(i + 1) == Some(&':') {
            out.push(Tok::Path);
            i += 2;
        } else {
            out.push(Tok::Punct(ch));
            i += 1;
        }
    }
    out
}

/// Why `path` (a full path, `*` for a glob) is refused, if it is.
fn refused(path: &[String]) -> Option<String> {
    let p: Vec<&str> = path.iter().map(String::as_str).collect();
    match p.as_slice() {
        ["web_time", ..] => Some("names web_time (read the clock through aegis_core::clock)".into()),
        ["std", "time"] | ["std", "time", "self" | "*", ..] => Some(format!("imports {} (use aegis_core::clock)", p.join("::"))),
        ["std", "time", name, ..] if CLOCK_NAMES.contains(name) => Some(format!("names std::time::{name} (use aegis_core::clock::{name})")),
        _ => None,
    }
}

/// Walks the path or use tree at `toks[*i]` under `prefix`; pushes each refused full path with its line.
fn walk_tree(toks: &[Tok], i: &mut usize, prefix: &mut Vec<String>, hits: &mut Vec<(usize, String)>) {
    let depth = prefix.len();
    match toks.get(*i) {
        Some(Tok::Punct('{')) => {
            *i += 1;
            while *i < toks.len() && toks[*i] != Tok::Punct('}') {
                walk_tree(toks, i, prefix, hits);
                if toks.get(*i) == Some(&Tok::Punct(',')) {
                    *i += 1;
                } else if toks.get(*i) != Some(&Tok::Punct('}')) {
                    break;
                }
            }
            if toks.get(*i) == Some(&Tok::Punct('}')) {
                *i += 1;
            }
        }
        Some(Tok::Punct('*')) => {
            *i += 1;
            prefix.push("*".into());
            check(prefix, line_near(toks, *i), hits);
        }
        Some(Tok::Ident(name, line)) => {
            let line = *line;
            prefix.push(name.clone());
            *i += 1;
            if toks.get(*i) == Some(&Tok::Path) {
                *i += 1;
                walk_tree(toks, i, prefix, hits);
            } else {
                check(prefix, line, hits);
            }
        }
        _ => {}
    }
    prefix.truncate(depth);
}

fn line_near(toks: &[Tok], i: usize) -> usize {
    toks[..i.min(toks.len())].iter().rev().find_map(|t| if let Tok::Ident(_, l) = t { Some(*l) } else { None }).unwrap_or(1)
}

fn check(path: &[String], line: usize, hits: &mut Vec<(usize, String)>) {
    if let Some(why) = refused(path) {
        hits.push((line, why));
    }
}

/// The refused clock paths of one source file's text: (line, reason).
fn violations(text: &str) -> (Vec<(usize, String)>, Vec<String>) {
    let mut s = strip(text);
    let mods = drop_test_items(&mut s);
    let toks = tokens(&s);
    let mut hits = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        // a path's first segment: not after `::`, or after a leading `::` (`use ::std::..`, `(::std::..`)
        let leading = i >= 2 && toks[i - 1] == Tok::Path && !matches!(&toks[i - 2], Tok::Ident(n, _) if n != "use");
        let starts = matches!(&toks[i], Tok::Ident(n, _) if n == "std" || n == "web_time") && (i == 0 || toks[i - 1] != Tok::Path || leading);
        if starts {
            walk_tree(&toks, &mut i, &mut Vec::new(), &mut hits);
        } else {
            i += 1;
        }
    }
    hits.sort();
    hits.dedup();
    (hits, mods)
}

/// The `src` folders the lint walks: every engine, kit and game crate but the CLI ones.
fn src_dirs(root: &Path) -> Vec<PathBuf> {
    let mut crates_dirs = vec![root.join("engine/crates")];
    for layer in ["kits", "games"] {
        let Ok(rd) = std::fs::read_dir(root.join(layer)) else { continue };
        for e in rd.flatten() {
            crates_dirs.push(e.path().join("crates"));
        }
    }
    let mut out = Vec::new();
    for d in crates_dirs {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !CLI_CRATES.contains(&name.as_str()) && e.path().join("src").is_dir() {
                out.push(e.path().join("src"));
            }
        }
    }
    out.sort();
    out
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Where the files of `mod name;` declared in `file` live: beside it for lib.rs / main.rs / mod.rs, else in its folder.
fn child_dir(file: &Path) -> PathBuf {
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let parent = file.parent().unwrap_or(Path::new(""));
    if ["lib", "main", "mod"].contains(&stem.as_ref()) {
        parent.to_path_buf()
    } else {
        parent.join(stem.as_ref())
    }
}

/// Every refused clock path in the tree, as `path:line: reason`.
fn lint_tree(root: &Path) -> Vec<String> {
    let allowed: BTreeSet<PathBuf> = ALLOWED.iter().map(|(p, _)| root.join(p)).collect();
    let mut found = Vec::new();
    let mut test_only: Vec<PathBuf> = Vec::new();
    for src in src_dirs(root) {
        let mut files = Vec::new();
        rs_files(&src, &mut files);
        for f in files {
            if f.starts_with(src.join("bin")) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&f) else { continue };
            let (hits, mods) = violations(&text);
            for m in mods {
                test_only.push(child_dir(&f).join(format!("{m}.rs")));
                test_only.push(child_dir(&f).join(&m));
            }
            found.push((f, hits));
        }
    }
    let mut out = Vec::new();
    for (f, hits) in found {
        if allowed.contains(&f) || test_only.iter().any(|t| f == *t || f.starts_with(t)) {
            continue;
        }
        let rel = f.strip_prefix(root).unwrap_or(&f).to_string_lossy().replace('\\', "/");
        out.extend(hits.into_iter().map(|(line, why)| format!("{rel}:{line}: {why}")));
    }
    out
}

#[test]
fn runtime_code_reads_the_clock_only_through_aegis_core_clock() {
    let root = root();
    for (p, _) in ALLOWED {
        assert!(root.join(p).is_file(), "an allowed site that no longer exists: {p} (drop it from ALLOWED)");
    }
    let found = lint_tree(&root);
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
    let dirs = src_dirs(&root);
    assert!(dirs.iter().any(|d| d.ends_with("aegis_ui/src")), "engine crates are walked");
    assert!(dirs.iter().any(|d| d.starts_with(root.join("kits"))), "kit crates are walked");
    assert!(dirs.iter().any(|d| d.starts_with(root.join("games"))), "game crates are walked");
    assert!(!dirs.iter().any(|d| CLI_CRATES.iter().any(|c| d.ends_with(format!("{c}/src")))), "CLI crates are not walked");
}
