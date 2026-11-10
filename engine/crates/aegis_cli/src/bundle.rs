//! `aegis bundle <manifest.json> [out=<dir>]` (WEBREL1-G1, `docs/specs/WEBREL1.md` 2.1, 3, 6.1): packs the files a web
//! build reads into its bundle parts. The manifest is the game's (it names what ships); this tool only follows it:
//!   {"root": "../..", "name": "<game id>", "exclude": ["*.md", "<path>"],
//!    "parts": [{"file": "<name>_1.pak", "include": ["<path>", ...], "exclude": ["<path>"]}, ...]}
//! `root` is relative to the manifest's folder; every other path is relative to `root`, `/`-separated, a file or a
//! folder (everything under it). An exclude is such a path or `*.<ext>` (any file with that ending); the top-level
//! list applies to every part. A file in two parts, an include that does not exist, or a part with no file is an
//! error. Each part is one file in `aegis_core::vfs`'s pack format (`Table::to_pack`), its keys the paths relative to
//! `root`, which the web start mounts as they are (`vfs::mount`). Beside the parts goes `bundle.json`, the list the
//! web start fetches first: {"name", "parts": [{"file", "files", "bytes"}]}, in load order (the first part before the
//! first frame, the later ones while the game runs).

use aegis_core::vfs::Table;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// One bundle part as written: its file name, file count and size in bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    pub file: String,
    pub files: usize,
    pub bytes: usize,
}

/// The list the web start reads (`bundle.json` beside the parts).
pub const INDEX: &str = "bundle.json";

fn strings(v: &Value, key: &str) -> Result<Vec<String>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(a)) => a.iter().map(|s| s.as_str().map(|s| s.trim_end_matches('/').to_string()).with_context(|| format!("{key}: every entry is a string"))).collect(),
        Some(_) => bail!("{key} is a list of strings"),
    }
}

/// Whether the root-relative key `rel` is under the path `p` (the file itself or a folder above it).
fn under(rel: &str, p: &str) -> bool {
    rel == p || (rel.starts_with(p) && rel.as_bytes().get(p.len()) == Some(&b'/'))
}

fn excluded(rel: &str, rules: &[String]) -> bool {
    rules.iter().any(|r| match r.strip_prefix('*') {
        Some(end) => rel.ends_with(end),
        None => under(rel, r),
    })
}

/// Every file under `dir` (or `dir` itself when it is a file) as root-relative keys, sorted.
fn files_of(root: &Path, rel: &str, out: &mut Vec<String>) -> Result<()> {
    let path = root.join(rel);
    if path.is_file() {
        out.push(rel.to_string());
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&path).with_context(|| format!("{}", path.display()))?.flatten().map(|e| e.path()).collect();
    entries.sort();
    for e in entries {
        let name = e.file_name().and_then(|n| n.to_str()).with_context(|| format!("{}: a name that is not UTF-8", e.display()))?;
        files_of(root, &format!("{rel}/{name}"), out)?;
    }
    Ok(())
}

/// The parts the manifest at `manifest` names, as (file name, table), in load order.
pub fn collect(manifest: &Path) -> Result<(String, Vec<(String, Table)>)> {
    let text = std::fs::read_to_string(manifest).with_context(|| format!("reading {}", manifest.display()))?;
    let m: Value = serde_json::from_str(&text).with_context(|| format!("parsing {}", manifest.display()))?;
    let root = manifest.parent().unwrap_or(Path::new(".")).join(m.get("root").and_then(Value::as_str).unwrap_or("."));
    let name = m.get("name").and_then(Value::as_str).unwrap_or("bundle").to_string();
    let common = strings(&m, "exclude")?;
    let parts = m.get("parts").and_then(Value::as_array).filter(|p| !p.is_empty()).context("parts: a list of at least one part")?;
    let mut seen: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut out = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        let file = p.get("file").and_then(Value::as_str).with_context(|| format!("part {}: no file name", i + 1))?.to_string();
        let mut rules = common.clone();
        rules.extend(strings(p, "exclude")?);
        let mut keys = Vec::new();
        for inc in strings(p, "include")? {
            if !root.join(&inc).exists() {
                bail!("{file}: include {inc} does not exist under {}", root.display());
            }
            files_of(&root, &inc, &mut keys)?;
        }
        let mut table = Table::new();
        for k in keys.into_iter().filter(|k| !excluded(k, &rules)) {
            if let Some(other) = seen.insert(k.clone(), file.clone()) {
                if other != file {
                    bail!("{k} is in two parts: {other} and {file}");
                }
                continue;
            }
            let bytes = std::fs::read(root.join(&k)).with_context(|| k.clone())?;
            table.insert(&k, bytes);
        }
        if table.is_empty() {
            bail!("{file}: no file (every include excluded?)");
        }
        out.push((file, table));
    }
    Ok((name, out))
}

/// Writes the parts and `bundle.json` into `out`.
pub fn write(manifest: &Path, out: &Path) -> Result<Vec<Written>> {
    let (name, parts) = collect(manifest)?;
    std::fs::create_dir_all(out).with_context(|| format!("{}", out.display()))?;
    let mut written = Vec::new();
    for (file, table) in parts {
        let pack = table.to_pack();
        std::fs::write(out.join(&file), &pack).with_context(|| format!("writing {file}"))?;
        written.push(Written { file, files: table.len(), bytes: pack.len() });
    }
    let index = json!({"name": name, "parts": written.iter().map(|w| json!({"file": w.file, "files": w.files, "bytes": w.bytes})).collect::<Vec<_>>()});
    std::fs::write(out.join(INDEX), serde_json::to_string_pretty(&index)? + "\n").context("writing bundle.json")?;
    Ok(written)
}

/// `aegis bundle <manifest.json> [out=<dir>]`: one line per part (default out: the working folder).
pub fn command(manifest: &str, out: &str) -> Result<()> {
    for w in write(Path::new(manifest), Path::new(out))? {
        println!("{}: {} files, {} bytes", w.file, w.files, w.bytes);
    }
    println!("{}", Path::new(out).join(INDEX).display());
    Ok(())
}
