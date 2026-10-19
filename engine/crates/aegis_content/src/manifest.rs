//! A game's manifest, `games/<id>/game.json` (docs/specs/SPLIT1.md section 5): the stable `id` (folder, crate prefix,
//! save folder, qualified ids), the display `title` and `subtitle` (window title, main menu), the version, the kit and
//! engine it needs, its worlds and start worlds, and the games it references. Written through `aegis content
//! manifest` (`set`), never by hand.

use crate::file::{read, stringify};
use crate::schema::is_id;
use aegis_core::json::{self, Dict};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const FILE: &str = "game.json";
/// The manifest's keys, in the order the file holds them.
pub const KEYS: [&str; 9] = ["id", "title", "subtitle", "version", "kit", "engine", "worlds", "start_worlds", "references"];

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub version: String,
    pub kit: String,
    pub engine: String,
    pub worlds: Vec<String>,
    pub start_worlds: Vec<String>,
    /// [{"game": "<id>", "version": "<semver range>"}] (reserved; read by nothing yet).
    pub references: Vec<Value>,
}

impl Manifest {
    pub fn from_dict(d: &Dict) -> Self {
        Self {
            id: json::s(d, "id"),
            title: json::s(d, "title"),
            subtitle: json::s(d, "subtitle"),
            version: json::s(d, "version"),
            kit: json::s(d, "kit"),
            engine: json::s(d, "engine"),
            worlds: json::sget(d, "worlds"),
            start_worlds: json::sget(d, "start_worlds"),
            references: json::aget(d, "references").to_vec(),
        }
    }

    pub fn to_dict(&self) -> Dict {
        let mut d = Dict::new();
        d.insert("id".into(), self.id.clone().into());
        d.insert("title".into(), self.title.clone().into());
        d.insert("subtitle".into(), self.subtitle.clone().into());
        d.insert("version".into(), self.version.clone().into());
        d.insert("kit".into(), self.kit.clone().into());
        d.insert("engine".into(), self.engine.clone().into());
        d.insert("worlds".into(), json::strs(&self.worlds));
        d.insert("start_worlds".into(), json::strs(&self.start_worlds));
        d.insert("references".into(), Value::Array(self.references.clone()));
        d
    }

    /// The manifest file; an error when it is missing or not an object.
    pub fn read(path: &Path) -> Result<Self, String> {
        let mut errors = Vec::new();
        match read(path, &mut errors) {
            Some(Value::Object(d)) => Ok(Self::from_dict(&d)),
            Some(_) => Err(format!("{}: not an object", path.display())),
            None => Err(errors.pop().unwrap_or_else(|| format!("{}: missing", path.display()))),
        }
    }

    /// What keeps the manifest from being valid: id, title, world ids, start worlds among the worlds.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !is_id(&self.id) {
            out.push(format!("game.json: id '{}' must be lowercase snake_case (a-z, 0-9, _).", self.id));
        }
        if self.title.trim().is_empty() {
            out.push("game.json: title is required.".into());
        }
        for w in self.worlds.iter().filter(|w| !is_id(w)) {
            out.push(format!("game.json: world '{w}' is not a world id."));
        }
        for w in self.start_worlds.iter().filter(|w| !self.worlds.contains(w)) {
            out.push(format!("game.json: start world '{w}' is not one of its worlds."));
        }
        out
    }

    /// Merges `patch` (manifest keys only) into the file at `path` (a new manifest when there is none), checks it and
    /// writes it; nothing is written when it has problems.
    pub fn set(path: &Path, patch: &Dict) -> Result<Self, Vec<String>> {
        let unknown: Vec<String> = patch.keys().filter(|k| !KEYS.contains(&k.as_str())).map(|k| format!("game.json has no key '{k}' (keys: {}).", KEYS.join(", "))).collect();
        if !unknown.is_empty() {
            return Err(unknown);
        }
        let mut d = if path.is_file() { Self::read(path).map_err(|e| vec![e])?.to_dict() } else { Self::default().to_dict() };
        for (k, v) in json::normalize_dict(patch) {
            d.insert(k, v);
        }
        let m = Self::from_dict(&d);
        let problems = m.problems();
        if !problems.is_empty() {
            return Err(problems);
        }
        std::fs::write(path, stringify(&Value::Object(m.to_dict()))).map_err(|e| vec![format!("{}: {e}", path.display())])?;
        Ok(m)
    }
}

/// The folder of game `id` from `start`: `start` or an ancestor when it is that game's folder (holds game.json and is
/// named `id`), else `games/<id>` under an ancestor.
pub fn find_game_dir(start: &Path, id: &str) -> Option<PathBuf> {
    start.ancestors().find_map(|d| {
        if d.file_name().is_some_and(|n| n == id) && d.join(FILE).is_file() {
            return Some(d.to_path_buf());
        }
        let g = d.join("games").join(id);
        g.join(FILE).is_file().then_some(g)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_checks_then_writes_and_finds_the_game() {
        let root = std::env::temp_dir().join(format!("aegis_manifest_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let game = root.join("games").join("demo");
        std::fs::create_dir_all(game.join("data")).unwrap();
        let path = game.join(FILE);
        let patch = |v: Value| v.as_object().cloned().unwrap();
        assert!(Manifest::set(&path, &patch(serde_json::json!({"id": "demo"}))).unwrap_err()[0].contains("title is required"));
        assert!(!path.exists());
        let m = Manifest::set(&path, &patch(serde_json::json!({"id": "demo", "title": "Demo", "worlds": ["one"], "start_worlds": ["one"]}))).unwrap();
        assert_eq!((m.title.as_str(), m.worlds.len()), ("Demo", 1));
        assert!(Manifest::set(&path, &patch(serde_json::json!({"colour": 1}))).unwrap_err()[0].contains("no key 'colour'"));
        assert!(Manifest::set(&path, &patch(serde_json::json!({"start_worlds": ["two"]}))).unwrap_err()[0].contains("start world 'two'"));
        let back = Manifest::read(&path).unwrap();
        assert_eq!(back.to_dict(), m.to_dict());
        assert_eq!(back.to_dict().keys().cloned().collect::<Vec<_>>(), KEYS.to_vec());
        assert_eq!(find_game_dir(&game.join("data"), "demo").as_deref(), Some(game.as_path()));
        assert_eq!(find_game_dir(&root, "demo").as_deref(), Some(game.as_path()));
        assert_eq!(find_game_dir(&root, "other"), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
