//! BASERULES1-M (docs/specs/BASERULES1.md 4.1): the `LOCKED` table cannot go stale. Every numeric `pub const` of
//! `rpgkit_rules/src` (a number or an array of numbers) is either listed in `LOCKED` (stays code, with why) or in
//! `PROMOTED` (the default of a ruleset row field); every entry of both names a constant that exists, in its file.
//! BASERULES1-M2 (FB-40): the scan recurses into subfolders and reads `pub(<scope>)`, `static`, slice-typed and
//! multi-line declarations too.

use rpgkit_rules::ruleset::{Numbers, LOCKED, PROMOTED};
use std::path::{Path, PathBuf};

const NUMERIC: [&str; 12] = ["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize", "f32", "f64"];

/// The numeric public constants and statics of every source file under `dir`, subfolders included (BASERULES1-M2,
/// FB-40): (file path relative to `dir`, with `/`, name). A declaration counts when it is `pub` or `pub(<scope>)`, `const`
/// or `static`, and its type is a number, an array of numbers or a slice of numbers (`&[f64]`, `&'static [i32]`); a
/// declaration whose type or `=` sits on the next lines is read up to its `=`.
fn numeric_consts_in(dir: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    collect_rs(dir, &mut files);
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let file = path.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let Some(rest) = declaration(line.trim_start()) else { continue };
            // a multi-line declaration: read on to the `=` (or the `;` of a bodyless one)
            let mut decl = rest.to_string();
            let mut j = i + 1;
            while !decl.contains('=') && !decl.contains(';') && j < lines.len() {
                decl.push(' ');
                decl.push_str(lines[j].trim());
                j += 1;
            }
            let Some((name, ty)) = decl.split_once(':') else { continue };
            if numeric_type(ty.split('=').next().unwrap_or("")) {
                out.push((file.clone(), name.trim().to_string()));
            }
        }
    }
    out
}

/// The text after `pub [(<scope>)] const|static ` of a declaration line, or None.
fn declaration(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("pub")?;
    let rest = match rest.strip_prefix('(') {
        Some(scoped) => scoped.split_once(')')?.1,
        None => rest,
    };
    let rest = rest.trim_start();
    rest.strip_prefix("const ").or_else(|| rest.strip_prefix("static "))
}

/// Whether a declared type is a number, an array of numbers or a slice of numbers.
fn numeric_type(ty: &str) -> bool {
    let ty = ty.trim().trim_start_matches('&').trim_start_matches("'static").trim();
    let base = ty.trim_start_matches('[').split([';', ']']).next().unwrap_or("").trim();
    NUMERIC.contains(&base)
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let path = e.unwrap().path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|x| x == "rs") {
            out.push(path);
        }
    }
}

/// The numeric public constants of `rpgkit_rules/src`.
fn numeric_consts() -> Vec<(String, String)> {
    numeric_consts_in(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"))
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

/// BASERULES1-M2 (FB-40): a numeric constant the old scan missed (a subfolder, `pub(crate)`, `static`, a slice type, a
/// declaration over two lines) is found, so an unlisted one fails the check above; non-numeric ones are not.
#[test]
fn the_scan_reads_nested_and_unusual_declarations() {
    let scratch = aegis_core::scratch::Scratch::new("locked_scan");
    let dir = scratch.path().to_path_buf();
    std::fs::create_dir_all(dir.join("deep/deeper")).unwrap();
    std::fs::write(dir.join("top.rs"), "pub const TOP: i32 = 1;\nconst PRIVATE: i32 = 2;\npub const NAME: &str = \"x\";\n").unwrap();
    std::fs::write(
        dir.join("deep/deeper/nested.rs"),
        "pub(crate) const SCOPED: f64 = 0.5;\npub static STATIC_N: u8 = 3;\npub const SLICE: &[f64] = &[1.0];\n\
         pub const ARRAY: [i32; 2] = [1, 2];\npub const SPLIT:\n    f64 = 2.0;\npub(super) static WORDS: &[&str] = &[];\n",
    )
    .unwrap();
    std::fs::write(dir.join("deep/notes.txt"), "pub const IGNORED: i32 = 9;\n").unwrap();
    let found = numeric_consts_in(&dir);
    let names: Vec<(&str, &str)> = found.iter().map(|(f, n)| (f.as_str(), n.as_str())).collect();
    assert_eq!(
        names,
        [
            ("deep/deeper/nested.rs", "SCOPED"),
            ("deep/deeper/nested.rs", "STATIC_N"),
            ("deep/deeper/nested.rs", "SLICE"),
            ("deep/deeper/nested.rs", "ARRAY"),
            ("deep/deeper/nested.rs", "SPLIT"),
            ("top.rs", "TOP"),
        ]
    );
    // the nested violation: not in LOCKED, not in PROMOTED, so the real check would name it
    let unlisted: Vec<&str> = names.iter().filter(|(f, n)| !LOCKED.iter().any(|l| l.name == *n && l.file == *f)).map(|(_, n)| *n).collect();
    assert!(unlisted.contains(&"SPLIT") && unlisted.contains(&"SCOPED"), "{unlisted:?}");
}
