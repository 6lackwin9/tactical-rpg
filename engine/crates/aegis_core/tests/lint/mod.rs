//! The source scanner behind the web runtime lints (`clock_lint.rs`, WEBREL1-E1a; `fs_lint.rs`, WEBREL1-E1b): std's
//! clock and file system compile on `wasm32-unknown-unknown` and fail only at run time there, so these tests walk the
//! non-test, non-CLI `src` of every engine, kit and game crate and refuse the std paths a `Rule` names.
//!
//! Not scanned: `tests/`, `benches/`, `examples/` (only `src/` is walked), `src/bin/` (CLI and dev tools), the CLI
//! crates a lint passes, `#[cfg(test)]` and `#[cfg(not(target_arch = "wasm32"))]` items (test-only and native-only
//! code never runs on the web; also the files of such a `mod name;`), comments and string or char literals.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What one lint refuses.
pub struct Rule {
    /// The first path segments that start a checked path (`std`, `web_time`).
    pub heads: &'static [&'static str],
    /// Why a full path (`*` for a glob) is refused, if it is.
    pub refused: fn(&[&str]) -> Option<String>,
    /// Methods refused when called with no arguments (`p.is_file()`: a std file system probe on a path).
    pub probes: &'static [&'static str],
}

/// The attributes whose item the scan skips, as they read after `normalise`.
const SKIPPED: &[&str] = &["#[cfg(test)]", "#[cfg(not_wasm32)]"];

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().expect("workspace root")
}

/// `text` with the native-only attribute spelled as one token (its string literal would be blanked by `strip`).
fn normalise(text: &str) -> String {
    text.replace("#[cfg(not(target_arch = \"wasm32\"))]", "#[cfg(not_wasm32)]")
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

/// Blanks every skipped (`SKIPPED`) item of stripped `text` (to its `;` or its closing brace, or, for a field, variant
/// or match arm, to its `,` or the enclosing close it did not open); returns the names of the skipped `mod name;` it
/// met (their files are skipped too).
fn drop_skipped_items(text: &mut String) -> Vec<String> {
    let mut mods = Vec::new();
    while let Some((start, attr)) = SKIPPED.iter().filter_map(|a| text.find(a).map(|at| (at, a.len()))).min() {
        let b = text.as_bytes();
        let mut i = start + attr;
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
        let item = &text[start + attr..i.min(b.len())];
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

/// Walks the path or use tree at `toks[*i]` under `prefix`; pushes each refused full path with its line.
fn walk_tree(rule: &Rule, toks: &[Tok], i: &mut usize, prefix: &mut Vec<String>, hits: &mut Vec<(usize, String)>) {
    let depth = prefix.len();
    match toks.get(*i) {
        Some(Tok::Punct('{')) => {
            *i += 1;
            while *i < toks.len() && toks[*i] != Tok::Punct('}') {
                walk_tree(rule, toks, i, prefix, hits);
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
            check(rule, prefix, line_near(toks, *i), hits);
        }
        Some(Tok::Ident(name, line)) => {
            let line = *line;
            prefix.push(name.clone());
            *i += 1;
            if toks.get(*i) == Some(&Tok::Path) {
                *i += 1;
                walk_tree(rule, toks, i, prefix, hits);
            } else {
                check(rule, prefix, line, hits);
            }
        }
        _ => {}
    }
    prefix.truncate(depth);
}

fn line_near(toks: &[Tok], i: usize) -> usize {
    toks[..i.min(toks.len())].iter().rev().find_map(|t| if let Tok::Ident(_, l) = t { Some(*l) } else { None }).unwrap_or(1)
}

fn check(rule: &Rule, path: &[String], line: usize, hits: &mut Vec<(usize, String)>) {
    let p: Vec<&str> = path.iter().map(String::as_str).collect();
    if let Some(why) = (rule.refused)(&p) {
        hits.push((line, why));
    }
}

/// The refused paths and probes of one source file's text: ((line, reason) sorted, skipped `mod` names).
pub fn violations(text: &str, rule: &Rule) -> (Vec<(usize, String)>, Vec<String>) {
    let mut s = strip(&normalise(text));
    let mods = drop_skipped_items(&mut s);
    let toks = tokens(&s);
    let mut hits = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        // a path's first segment: not after `::`, or after a leading `::` (`use ::std::..`, `(::std::..`)
        let leading = i >= 2 && toks[i - 1] == Tok::Path && !matches!(&toks[i - 2], Tok::Ident(n, _) if n != "use");
        let starts = matches!(&toks[i], Tok::Ident(n, _) if rule.heads.contains(&n.as_str())) && (i == 0 || toks[i - 1] != Tok::Path || leading);
        if starts {
            walk_tree(rule, &toks, &mut i, &mut Vec::new(), &mut hits);
            continue;
        }
        // a no-argument probe method: `.name()`
        if let (Tok::Punct('.'), Some(Tok::Ident(name, line)), Some(Tok::Punct('(')), Some(Tok::Punct(')'))) = (&toks[i], toks.get(i + 1), toks.get(i + 2), toks.get(i + 3)) {
            if rule.probes.contains(&name.as_str()) {
                hits.push((*line, format!("calls .{name}() (a std file system probe; use aegis_core::vfs)")));
            }
        }
        i += 1;
    }
    hits.sort();
    hits.dedup();
    (hits, mods)
}

/// The `src` folders a lint walks: every engine, kit and game crate but the ones in `cli`.
pub fn src_dirs(root: &Path, cli: &[&str]) -> Vec<PathBuf> {
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
            if !cli.contains(&name.as_str()) && e.path().join("src").is_dir() {
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
pub fn child_dir(file: &Path) -> PathBuf {
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let parent = file.parent().unwrap_or(Path::new(""));
    if ["lib", "main", "mod"].contains(&stem.as_ref()) {
        parent.to_path_buf()
    } else {
        parent.join(stem.as_ref())
    }
}

/// Every refused path in the tree, as `path:line: reason`; files in `allowed` (workspace-relative) are not reported.
pub fn lint_tree(root: &Path, rule: &Rule, allowed: &[(&str, &str)], cli: &[&str]) -> Vec<String> {
    let allowed: BTreeSet<PathBuf> = allowed.iter().map(|(p, _)| root.join(p)).collect();
    let mut found = Vec::new();
    let mut skipped: Vec<PathBuf> = Vec::new();
    for src in src_dirs(root, cli) {
        let mut files = Vec::new();
        rs_files(&src, &mut files);
        for f in files {
            if f.starts_with(src.join("bin")) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&f) else { continue };
            let (hits, mods) = violations(&text, rule);
            for m in mods {
                skipped.push(child_dir(&f).join(format!("{m}.rs")));
                skipped.push(child_dir(&f).join(&m));
            }
            found.push((f, hits));
        }
    }
    let mut out = Vec::new();
    for (f, hits) in found {
        if allowed.contains(&f) || skipped.iter().any(|t| f == *t || f.starts_with(t)) {
            continue;
        }
        let rel = f.strip_prefix(root).unwrap_or(&f).to_string_lossy().replace('\\', "/");
        out.extend(hits.into_iter().map(|(line, why)| format!("{rel}:{line}: {why}")));
    }
    out
}
