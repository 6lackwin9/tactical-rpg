//! The content writer (ContentCatalog.upsert / WorldCatalog.upsert, without Godot): an entry is given in its stored
//! shape, checked by the game, put into its file (replacing the entry with its id, or `replace_id`, else appended)
//! and kept only when it adds no blocking problem to the data folder, else the file is put back as it was. The game's
//! `Hooks` say which kinds exist, where they live, how an entry is checked and how a file is written. A world's
//! manifest (world.json) is written the same way through `set_world` (WORLDW1), its shape given by the game. The game
//! manifest (game.json) is written through `set_manifest`: the engine keys plus the extra keys the game registers
//! (FB-110, E2.2-S2), checked by the game.

use crate::file::raw_entries;
use crate::manifest::Manifest;
use crate::schema::sget;
use crate::world::manifest_path;
use aegis_core::json::{self, Dict};
use std::path::{Path, PathBuf};

/// What the writer asks of the game.
pub trait Hooks {
    /// The kinds an entry may have in the base catalog (`world` None) or in a world.
    fn kinds(&self, world: Option<&str>) -> &'static [&'static str];
    /// The file the entries of `kind` live in.
    fn file_of(&self, data_dir: &Path, world: Option<&str>, kind: &str) -> PathBuf;
    /// The entry (stored shape, numbers JsonUtil-normalized) as the schema sees it, or its problems.
    fn validate(&self, world: Option<&str>, kind: &str, entry: &Dict) -> Result<Dict, Vec<String>>;
    /// The file text for `entries` (stored shape) of `kind`.
    fn file_text(&self, data_dir: &Path, world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<String, String>;
    /// Every hard problem of the data folder (load problems, broken references, the game's content rules).
    fn check(&self, data_dir: &Path) -> Vec<String>;
    /// The problems that refuse a write (a write may not add one); `check` unless the game blocks on less (Godot's
    /// worlds take unresolved references, since worlds are full of mutual ones).
    fn blocking(&self, data_dir: &Path) -> Vec<String> {
        self.check(data_dir)
    }
    /// The soft problems of the data folder (never blocking a write); none unless the game has lints.
    fn warnings(&self, _data_dir: &Path) -> Vec<String> {
        Vec::new()
    }
    /// The text of world `world`'s manifest (data/worlds/<world>/world.json) after a write of `patch`: the fields
    /// merged over the stored manifest (over the schema defaults for a new world), checked, in the stored shape
    /// (`full`: every field written, else fields the stored file leaves out stay out while at their default). None by
    /// default: a game without world manifests refuses the write.
    fn world_manifest_text(&self, _data_dir: &Path, _world: &str, _patch: &Dict, _full: bool) -> Result<String, Vec<String>> {
        Err(vec!["This game has no world manifests.".into()])
    }
    /// The problems a world manifest write may not add; `blocking` unless the game checks manifests on top.
    fn world_blocking(&self, data_dir: &Path) -> Vec<String> {
        self.blocking(data_dir)
    }
    /// FB-110 (E2.2-S2): the keys the game manifest (game.json) may hold beyond the engine's `manifest::KEYS` (the
    /// game's own game-wide settings), in the order the file holds them after the engine keys; none by default.
    fn manifest_keys(&self) -> &'static [&'static str] {
        &[]
    }
    /// What keeps those keys from being valid in `manifest` (the game manifest object as it would be written); none by
    /// default.
    fn manifest_problems(&self, _manifest: &Dict) -> Vec<String> {
        Vec::new()
    }
}

/// What an upsert or remove did.
pub struct Outcome {
    pub file: PathBuf,
    pub replaced: bool,
}

/// Validates `entry` (stored shape), replaces the entry with its id (or `replace_id`) or appends it, writes the file,
/// and keeps it only when it adds no blocking problem to the data folder.
pub fn upsert(hooks: &impl Hooks, data_dir: &Path, world: Option<&str>, kind: &str, entry: &Dict, replace_id: Option<&str>) -> Result<Outcome, Vec<String>> {
    let known = hooks.kinds(world);
    if !known.contains(&kind) {
        return Err(vec![format!("Unknown kind '{kind}' (one of {}).", known.join(", "))]);
    }
    if let Some(w) = world {
        if !manifest_path(data_dir, w).is_file() {
            return Err(vec![format!("Unknown world '{w}'.")]);
        }
    }
    let entry = json::normalize_dict(entry);
    let full = hooks.validate(world, kind, &entry)?;
    let id = sget(&full, "id", "");
    let target = replace_id.unwrap_or(&id).to_string();
    let file = hooks.file_of(data_dir, world, kind);
    let mut entries = raw_entries(&file);
    let at = entries.iter().position(|e| sget(e, "id", "") == target);
    if replace_id.is_some() && at.is_none() {
        return Err(vec![format!("No {kind} entry '{target}' to replace.")]);
    }
    if target != id && entries.iter().any(|e| sget(e, "id", "") == id) {
        return Err(vec![format!("A {kind} entry '{id}' already exists.")]);
    }
    match at {
        Some(i) => entries[i] = entry,
        None => entries.push(entry),
    }
    let outcome = Outcome { file, replaced: at.is_some() };
    commit(hooks, data_dir, world, kind, &entries)?;
    Ok(outcome)
}

/// Removes the entry `id` of `kind` (kept only when it adds no blocking problem).
pub fn remove(hooks: &impl Hooks, data_dir: &Path, world: Option<&str>, kind: &str, id: &str) -> Result<Outcome, Vec<String>> {
    let file = hooks.file_of(data_dir, world, kind);
    let mut entries = raw_entries(&file);
    let Some(i) = entries.iter().position(|e| sget(e, "id", "") == id) else { return Err(vec![format!("No {kind} entry '{id}'.")]) };
    entries.remove(i);
    let outcome = Outcome { file, replaced: true };
    commit(hooks, data_dir, world, kind, &entries)?;
    Ok(outcome)
}

/// Creates or updates world `world`'s manifest (data/worlds/<world>/world.json, the world's folder made when new):
/// `patch` merged in by the game (`Hooks::world_manifest_text`), kept only when it adds no problem
/// (`Hooks::world_blocking`) to the data folder, else the file (and a folder it made) is taken back. `replaced` is
/// true when the world existed.
pub fn set_world(hooks: &impl Hooks, data_dir: &Path, world: &str, patch: &Dict, full: bool) -> Result<Outcome, Vec<String>> {
    if !crate::schema::is_id(world) {
        return Err(vec![format!("World '{world}' must be a world id (lowercase snake_case).")]);
    }
    let path = manifest_path(data_dir, world);
    let existed = path.is_file();
    let text = hooks.world_manifest_text(data_dir, world, patch, full)?;
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let new_folder = !folder.is_dir();
    let old_problems = hooks.world_blocking(data_dir);
    std::fs::create_dir_all(&folder).map_err(|e| vec![format!("{}: {e}", folder.display())])?;
    write_checked(&path, &text, &old_problems, || hooks.world_blocking(data_dir)).inspect_err(|_| {
        if new_folder {
            std::fs::remove_dir(&folder).ok();
        }
    })?;
    Ok(Outcome { file: path, replaced: existed })
}

/// FB-110 (E2.2-S2): merges `patch` into the game manifest at `path` (`Manifest::set_with`): the engine keys and the
/// game's extra keys (`Hooks::manifest_keys`), checked by the engine and the game (`Hooks::manifest_problems`); nothing
/// is written when it has problems. An extra key the file holds that the game does not register (a hand-edited typo)
/// is a problem too, so it is never kept silently; a patch setting it to null removes it.
pub fn set_manifest(hooks: &impl Hooks, path: &Path, patch: &Dict) -> Result<Manifest, Vec<String>> {
    let registered = hooks.manifest_keys();
    Manifest::set_with(path, patch, registered, |m| {
        let mut problems: Vec<String> = m
            .keys()
            .filter(|k| !crate::manifest::KEYS.contains(&k.as_str()) && !registered.contains(&k.as_str()))
            .map(|k| format!("game.json: key '{k}' is not a game.json key (registered: {}); set it to null to remove it.", if registered.is_empty() { "none".to_string() } else { registered.join(", ") }))
            .collect();
        problems.extend(hooks.manifest_problems(m));
        problems
    })
}

/// Writes the file, then checks the data folder again: new problems put the file back.
fn commit(hooks: &impl Hooks, data_dir: &Path, world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<(), Vec<String>> {
    let path = hooks.file_of(data_dir, world, kind);
    let old_problems = hooks.blocking(data_dir);
    let text = hooks.file_text(data_dir, world, kind, entries).map_err(|e| vec![e])?;
    write_checked(&path, &text, &old_problems, || hooks.blocking(data_dir))
}

/// Writes `text` to `path`; when `problems()` then lists one that `old_problems` does not, the file is put back as it
/// was (removed when it was new) and the new problems are the error.
fn write_checked(path: &Path, text: &str, old_problems: &[String], problems: impl Fn() -> Vec<String>) -> Result<(), Vec<String>> {
    let before = std::fs::read_to_string(path).ok();
    std::fs::write(path, text).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    let new: Vec<String> = problems().into_iter().filter(|p| !old_problems.contains(p)).collect();
    if !new.is_empty() {
        match before {
            Some(t) => std::fs::write(path, t).ok(),
            None => std::fs::remove_file(path).ok(),
        };
        return Err(new);
    }
    Ok(())
}
