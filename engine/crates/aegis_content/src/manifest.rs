//! A game's manifest, `games/<id>/game.json` (docs/specs/SPLIT1.md section 5): the stable `id` (folder, crate prefix,
//! save folder, qualified ids), the display `title` and `subtitle` (window title, main menu), the version, the kit and
//! engine it needs, its worlds and start worlds, and the games it references. Written through `aegis content
//! manifest` (`set`), never by hand. `icon` (ICON1, optional): the window icon, a PNG path over the asset layers.
//! `canvas` and `window_sizes` (LAYOUT720-E5, optional): how the game's canvas relates to the window and the window sizes
//! it offers; held as written, as the shell reads them (a bad value is a notice there and the default).

use crate::file::{read, stringify};
use crate::schema::is_id;
use aegis_core::json::{self, Dict};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const FILE: &str = "game.json";
/// The manifest's keys, in the order the file holds them.
pub const KEYS: [&str; 12] = ["id", "title", "subtitle", "icon", "canvas", "window_sizes", "version", "kit", "engine", "worlds", "start_worlds", "references"];

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    /// The window icon: a PNG path relative to the asset layers; "" none, and the file then has no such key.
    pub icon: String,
    /// The canvas policy (`{"policy", "w", "h"}`) as written; null: the default, and the file then has no such key.
    pub canvas: Value,
    /// The window sizes offered (a list of `WxH` strings) as written; null: the default, and the file then has no such key.
    pub window_sizes: Value,
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
            icon: json::s(d, "icon"),
            // Reading a file makes every number a float (`file::read`); these go on as written, whole numbers whole.
            canvas: d.get("canvas").map_or(Value::Null, json::normalize),
            window_sizes: d.get("window_sizes").map_or(Value::Null, json::normalize),
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
        if !self.icon.is_empty() {
            d.insert("icon".into(), self.icon.clone().into());
        }
        for (key, value) in [("canvas", &self.canvas), ("window_sizes", &self.window_sizes)] {
            if !value.is_null() {
                d.insert(key.into(), value.clone());
            }
        }
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

    /// What keeps the manifest from being valid: id, title, the icon's path form, world ids, start worlds among the worlds.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !is_id(&self.id) {
            out.push(format!("game.json: id '{}' must be lowercase snake_case (a-z, 0-9, _).", self.id));
        }
        if self.title.trim().is_empty() {
            out.push("game.json: title is required.".into());
        }
        if !self.icon.is_empty() && (self.icon.starts_with(['/', '\\']) || self.icon.contains(':') || self.icon.split(['/', '\\']).any(|part| part == "..") || !self.icon.ends_with(".png")) {
            out.push(format!("game.json: icon '{}' must be a .png path inside the asset layers.", self.icon));
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
        let mut problems = m.problems();
        problems.extend(path.parent().and_then(|g| kit_problem(g, &m.kit)));
        if !problems.is_empty() {
            return Err(problems);
        }
        std::fs::write(path, stringify(&Value::Object(m.to_dict()))).map_err(|e| vec![format!("{}: {e}", path.display())])?;
        Ok(m)
    }
}

/// The workspace of the game in `game_dir`: its first ancestor holding `engine/assets/` (ASSETORG2).
pub fn workspace_of(game_dir: &Path) -> Option<PathBuf> {
    game_dir.ancestors().skip(1).find(|d| d.join("engine/assets").is_dir()).map(Path::to_path_buf)
}

/// None, or why `kit` cannot be the kit of the game in `game_dir`: "" (no kit) or a folder `kits/<kit>/` holding
/// kit.json in the game's workspace.
pub fn kit_problem(game_dir: &Path, kit: &str) -> Option<String> {
    if kit.is_empty() {
        return None;
    }
    if !is_id(kit) {
        return Some(format!("game.json: kit '{kit}' must be a kit id (lowercase snake_case)."));
    }
    let found = workspace_of(game_dir).is_some_and(|ws| ws.join("kits").join(kit).join(KIT_FILE).is_file());
    (!found).then(|| format!("game.json: kit '{kit}' has no kits/{kit}/{KIT_FILE}."))
}

pub const KIT_FILE: &str = "kit.json";

/// Merges `patch` into the kit manifest at `path` (`kits/<id>/kit.json`; a new one when there is none), checks it
/// (`aegis_core::assets::kit_problems`: the id is the folder name) and writes it in `KIT_KEYS` order.
pub fn set_kit(path: &Path, patch: &Dict) -> Result<Dict, Vec<String>> {
    let folder = path.parent().and_then(Path::file_name).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let mut d: Dict = if path.is_file() {
        let mut errors = Vec::new();
        match read(path, &mut errors) {
            Some(Value::Object(o)) => o,
            _ => return Err(vec![errors.pop().unwrap_or_else(|| format!("{}: not an object", path.display()))]),
        }
    } else {
        Dict::new()
    };
    for (k, v) in json::normalize_dict(patch) {
        d.insert(k, v);
    }
    let problems = aegis_core::assets::kit_problems(&d, &folder);
    if !problems.is_empty() {
        return Err(problems);
    }
    let ordered: Dict = aegis_core::assets::KIT_KEYS.iter().map(|k| (k.to_string(), d.get(*k).cloned().unwrap_or_else(|| Value::from("")))).collect();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| vec![format!("{}: {e}", dir.display())])?;
    }
    std::fs::write(path, stringify(&Value::Object(ordered.clone()))).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    Ok(ordered)
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
        // Every key but the optional ones, which this manifest does not set.
        assert_eq!(back.to_dict().keys().cloned().collect::<Vec<_>>(), KEYS.iter().filter(|k| !["icon", "canvas", "window_sizes"].contains(*k)).copied().collect::<Vec<_>>());
        assert_eq!(find_game_dir(&game.join("data"), "demo").as_deref(), Some(game.as_path()));
        assert_eq!(find_game_dir(&root, "demo").as_deref(), Some(game.as_path()));
        assert_eq!(find_game_dir(&root, "other"), None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_kit_must_exist_and_its_manifest_names_its_folder() {
        let root = std::env::temp_dir().join(format!("aegis_kit_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("engine/assets")).unwrap();
        let game = root.join("games").join("demo");
        std::fs::create_dir_all(&game).unwrap();
        let path = game.join(FILE);
        let patch = |v: Value| v.as_object().cloned().unwrap();
        Manifest::set(&path, &patch(serde_json::json!({"id": "demo", "title": "Demo"}))).unwrap();
        assert_eq!(Manifest::set(&path, &patch(serde_json::json!({"kit": "rpg"}))).unwrap_err(), ["game.json: kit 'rpg' has no kits/rpg/kit.json."]);
        let kit = root.join("kits/rpg").join(KIT_FILE);
        assert!(set_kit(&kit, &patch(serde_json::json!({"id": "rpg", "title": "RPG kit", "version": "0.1"}))).is_err(), "semver and engine refused");
        assert!(!kit.exists());
        let written = set_kit(&kit, &patch(serde_json::json!({"engine": ">=0.1", "id": "rpg", "title": "RPG kit", "version": "0.1.0"}))).unwrap();
        assert_eq!(written.keys().cloned().collect::<Vec<_>>(), aegis_core::assets::KIT_KEYS.to_vec(), "written in key order");
        assert_eq!(Manifest::set(&path, &patch(serde_json::json!({"kit": "rpg"}))).unwrap().kit, "rpg");
        assert!(set_kit(&kit, &patch(serde_json::json!({"id": "other"}))).unwrap_err()[0].contains("folder name 'rpg'"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
