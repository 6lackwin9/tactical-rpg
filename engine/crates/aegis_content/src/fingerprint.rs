//! Content fingerprint of a data folder (PLAYTEST1 ContentFingerprint): every `*.json` under the root, by sorted
//! relative path ("/" separators), hashed with SHA-256 cut to 12 hex characters; `total` is the same cut of the
//! digest of the joined "<path>:<hash>\n" lines. A recording keeps the fingerprint of the data it was played on, so a
//! replay can say which files changed since (staleness warns, it never blocks).

use aegis_core::json::{self, Dict};
use aegis_core::sha256;
use serde_json::{json, Value};
use std::path::Path;

pub const ALGO: &str = "sha256-12";

fn cut(data: &[u8]) -> String {
    sha256::hex(data)[..12].to_string()
}

fn collect(dir: &Path, prefix: &str, out: &mut Vec<(String, String)>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    for e in read.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
        if p.is_dir() {
            collect(&p, &rel, out);
        } else if p.extension().is_some_and(|x| x == "json") {
            if let Ok(bytes) = std::fs::read(&p) {
                out.push((rel, cut(&bytes)));
            }
        }
    }
}

/// {algo, total, files: {path: hash}} of the data folder at `root`.
pub fn of(root: &Path) -> Value {
    let mut files = Vec::new();
    collect(root, "", &mut files);
    files.sort();
    let lines: String = files.iter().map(|(p, h)| format!("{p}:{h}\n")).collect();
    let map: Dict = files.into_iter().map(|(p, h)| (p, Value::from(h))).collect();
    json!({"algo": ALGO, "total": cut(lines.as_bytes()), "files": map})
}

/// The paths added, removed and changed from fingerprint `old` to `new` (empty lists when the totals agree).
pub fn changes(old: &Value, new: &Value) -> Value {
    let (a, b) = (json::dget(json::dict(old), "files"), json::dget(json::dict(new), "files"));
    if json::s(json::dict(old), "total") == json::s(json::dict(new), "total") {
        return json!({"added": [], "removed": [], "changed": []});
    }
    let added: Vec<&String> = b.keys().filter(|k| !a.contains_key(*k)).collect();
    let removed: Vec<&String> = a.keys().filter(|k| !b.contains_key(*k)).collect();
    let changed: Vec<&String> = a.iter().filter(|(k, h)| b.get(*k).is_some_and(|x| x != *h)).map(|(k, _)| k).collect();
    json!({"added": added, "removed": removed, "changed": changed})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_changed_file_changes_the_total_and_is_listed() {
        let dir = std::env::temp_dir().join(format!("aegis_fingerprint_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("catalog")).unwrap();
        std::fs::write(dir.join("catalog/a.json"), "{}").unwrap();
        std::fs::write(dir.join("b.json"), "[]").unwrap();
        std::fs::write(dir.join("notes.txt"), "not content").unwrap();
        let before = of(&dir);
        assert_eq!(json::dget(json::dict(&before), "files").keys().cloned().collect::<Vec<_>>(), vec!["b.json", "catalog/a.json"]);
        assert_eq!(changes(&before, &of(&dir))["changed"], json!([]));
        std::fs::write(dir.join("catalog/a.json"), "{\"x\": 1}").unwrap();
        let after = of(&dir);
        assert_ne!(before["total"], after["total"]);
        assert_eq!(changes(&before, &after), json!({"added": [], "removed": [], "changed": ["catalog/a.json"]}));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
