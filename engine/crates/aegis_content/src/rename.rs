//! ENGINE-38 (LORE-RENAME1): find and rename an id across a game's data, its seeds and its asset paths, so a lore
//! revision (a renamed unit, character, item, skill, quest, place, encounter or flag) is one command.
//!
//! A reference is a JSON string (a value or an object key) in a `.json` file under the data folder (catalogs, worlds,
//! seeds) that is the id, or names it as one of its `/` or `:` separated parts (a path such as `rogue/<id>` or
//! `portraits/<id>.png`, a qualified `<world>:<id>`); a part matches on its whole text or on its stem before the first
//! `.`. An asset reference is a file or folder under an asset root whose name (or stem) is the id. The text of a JSON
//! file is changed only inside the strings that match, so the rest of each file stays byte for byte as it was. The id
//! is matched whole and case sensitive; prose (a string with other words around the id) never matches.

use std::path::{Path, PathBuf};

/// One place that names the id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    /// The file (a JSON file, or the asset file or folder itself).
    pub path: PathBuf,
    /// 1-based line of the string in a JSON file (0 for an asset path).
    pub line: usize,
    /// The string as written (without its quotes), or the asset's name.
    pub text: String,
}

/// What a rename changes (or would change, in a dry run).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Every JSON file it rewrites, with how many strings change in it.
    pub files: Vec<(PathBuf, usize)>,
    /// Every asset file or folder it moves, from and to.
    pub moves: Vec<(PathBuf, PathBuf)>,
}

/// Whether `id` is a plain id: lowercase letters, digits, `_`, `-` and `.` (a flag key such as `ward.gate_seen`).
pub fn plain_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'))
}

/// `s` with each part that names `old` (whole, or its stem before the first `.`) renamed to `new`; None when no part
/// does.
pub fn rename_in(s: &str, old: &str, new: &str) -> Option<String> {
    if s == old {
        return Some(new.to_string());
    }
    if !s.contains(old) || s.contains(char::is_whitespace) {
        return None;
    }
    let mut out = String::with_capacity(s.len() + new.len());
    let mut changed = false;
    let mut rest = s;
    loop {
        let cut = rest.find(['/', ':']).unwrap_or(rest.len());
        let part = &rest[..cut];
        if part == old {
            out.push_str(new);
            changed = true;
        } else if let Some(tail) = part.strip_prefix(old).filter(|t| t.starts_with('.')) {
            out.push_str(new);
            out.push_str(tail);
            changed = true;
        } else {
            out.push_str(part);
        }
        if cut == rest.len() {
            break;
        }
        out.push_str(&rest[cut..cut + 1]);
        rest = &rest[cut + 1..];
    }
    changed.then_some(out)
}

/// The string literals of JSON text `text`: (byte start of the opening quote, byte end after the closing quote, the
/// decoded string). Text that is not JSON yields what it can; a file that does not parse is never rewritten (`plan`).
fn strings(text: &str) -> Vec<(usize, usize, String)> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            let start = i;
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            let end = (i + 1).min(b.len());
            if let Ok(s) = serde_json::from_str::<String>(&text[start..end]) {
                out.push((start, end, s));
            }
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

/// JSON text `text` with every string naming `old` renamed (`rename_in`), and how many strings changed; the rest of the
/// text as it was.
pub fn rename_text(text: &str, old: &str, new: &str) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut n = 0;
    for (start, end, s) in strings(text) {
        if let Some(r) = rename_in(&s, old, new) {
            out.push_str(&text[last..start]);
            out.push_str(&serde_json::to_string(&r).unwrap_or_default());
            last = end;
            n += 1;
        }
    }
    out.push_str(&text[last..]);
    (out, n)
}

/// Every `.json` file under `dir` (sorted, recursively).
fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(dir, &mut |p| {
        if p.extension().is_some_and(|e| e == "json") {
            out.push(p.to_path_buf());
        }
    });
    out.sort();
    out
}

fn walk(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let Ok(mut entries) = aegis_core::vfs::list_dir(dir) else { return };
    entries.sort();
    for p in entries {
        f(&p);
        if aegis_core::vfs::is_dir(&p) {
            walk(&p, f);
        }
    }
}

/// Whether an asset's file or folder name names `id` (whole, or its stem before the first `.`).
fn names(name: &str, id: &str) -> bool {
    name == id || name.strip_prefix(id).is_some_and(|rest| rest.starts_with('.'))
}

/// The asset files and folders under `roots` whose own name names `id` (a folder's contents move with it, so only the
/// outermost is listed).
fn asset_hits(roots: &[PathBuf], id: &str) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for root in roots {
        walk(root, &mut |p| {
            let named = p.file_name().and_then(|n| n.to_str()).is_some_and(|n| names(n, id));
            if named && !out.iter().any(|o| p.starts_with(o)) {
                out.push(p.to_path_buf());
            }
        });
    }
    out
}

/// `aegis content refs`: every string naming `id` in the data folder's JSON files (in file order), then every asset
/// file or folder under `asset_roots` named by it.
pub fn refs(data_dir: &Path, asset_roots: &[PathBuf], id: &str) -> Vec<Ref> {
    let mut out = Vec::new();
    for path in json_files(data_dir) {
        let Ok(text) = aegis_core::vfs::read_to_string(&path) else { continue };
        for (start, _, s) in strings(&text) {
            if rename_in(&s, id, "\u{1}").is_some() {
                let line = text[..start].matches('\n').count() + 1;
                out.push(Ref { path: path.clone(), line, text: s });
            }
        }
    }
    for p in asset_hits(asset_roots, id) {
        let text = p.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
        out.push(Ref { path: p, line: 0, text });
    }
    out
}

/// What renaming `old` to `new` changes: the JSON files with their counts (a file that does not parse as JSON is left
/// out), and the asset moves (each name's `old` part renamed, in its own folder). Writes nothing.
pub fn plan(data_dir: &Path, asset_roots: &[PathBuf], old: &str, new: &str) -> Plan {
    let mut p = Plan::default();
    for path in json_files(data_dir) {
        let Ok(text) = aegis_core::vfs::read_to_string(&path) else { continue };
        if serde_json::from_str::<serde_json::Value>(&text).is_err() {
            continue;
        }
        let (_, n) = rename_text(&text, old, new);
        if n > 0 {
            p.files.push((path, n));
        }
    }
    for from in asset_hits(asset_roots, old) {
        let name = from.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let to = from.with_file_name(format!("{new}{}", &name[old.len()..]));
        p.moves.push((from, to));
    }
    p
}

/// Applies `plan` (renaming `old` to `new`): rewrites its JSON files and makes its moves. On the first failure, or when
/// `problems()` then lists a problem it did not list before (`before`), every file and move is put back and the new
/// problems (or the failure) are the error.
pub fn apply(plan: &Plan, old: &str, new: &str, before: &[String], problems: impl Fn() -> Vec<String>) -> Result<(), Vec<String>> {
    let mut written: Vec<(PathBuf, String)> = Vec::new();
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    let undo = |written: &[(PathBuf, String)], moved: &[(PathBuf, PathBuf)]| {
        for (from, to) in moved.iter().rev() {
            let _ = std::fs::rename(to, from);
        }
        for (path, text) in written {
            let _ = std::fs::write(path, text);
        }
    };
    for (path, _) in &plan.files {
        let result = aegis_core::vfs::read_to_string(path).and_then(|text| {
            let (renamed, _) = rename_text(&text, old, new);
            std::fs::write(path, renamed).map(|()| text)
        });
        match result {
            Ok(text) => written.push((path.clone(), text)),
            Err(e) => {
                undo(&written, &moved);
                return Err(vec![format!("{}: {e}", path.display())]);
            }
        }
    }
    for (from, to) in &plan.moves {
        if aegis_core::vfs::exists(to) {
            undo(&written, &moved);
            return Err(vec![format!("{} already exists", to.display())]);
        }
        if let Err(e) = std::fs::rename(from, to) {
            undo(&written, &moved);
            return Err(vec![format!("{} -> {}: {e}", from.display(), to.display())]);
        }
        moved.push((from.clone(), to.clone()));
    }
    let new_problems: Vec<String> = problems().into_iter().filter(|p| !before.contains(p)).collect();
    if !new_problems.is_empty() {
        undo(&written, &moved);
        return Err(new_problems);
    }
    Ok(())
}
