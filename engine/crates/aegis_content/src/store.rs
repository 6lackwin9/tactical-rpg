//! The content writer (ContentCatalog.upsert / WorldCatalog.upsert, without Godot): an entry is given in its stored
//! shape, checked by the game, put into its file (replacing the entry with its id, or `replace_id`, else appended)
//! and kept only when the whole data folder still checks cleanly, else the file is put back as it was. The game's
//! `Hooks` say which kinds exist, where they live, how an entry is checked and how a file is written.

use crate::file::raw_entries;
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
    /// Every problem of the data folder (load problems, broken references).
    fn check(&self, data_dir: &Path) -> Vec<String>;
}

/// What an upsert or remove did.
pub struct Outcome {
    pub file: PathBuf,
    pub replaced: bool,
}

/// Validates `entry` (stored shape), replaces the entry with its id (or `replace_id`) or appends it, writes the file,
/// and keeps it only when the whole data folder still checks cleanly.
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

/// Removes the entry `id` of `kind` (no reference check beyond the data folder still checking cleanly).
pub fn remove(hooks: &impl Hooks, data_dir: &Path, world: Option<&str>, kind: &str, id: &str) -> Result<Outcome, Vec<String>> {
    let file = hooks.file_of(data_dir, world, kind);
    let mut entries = raw_entries(&file);
    let Some(i) = entries.iter().position(|e| sget(e, "id", "") == id) else { return Err(vec![format!("No {kind} entry '{id}'.")]) };
    entries.remove(i);
    let outcome = Outcome { file, replaced: true };
    commit(hooks, data_dir, world, kind, &entries)?;
    Ok(outcome)
}

/// Writes the file, then checks the data folder again: new problems put the file back.
fn commit(hooks: &impl Hooks, data_dir: &Path, world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<(), Vec<String>> {
    let path = hooks.file_of(data_dir, world, kind);
    let before = std::fs::read_to_string(&path).ok();
    let old_problems = hooks.check(data_dir);
    let text = hooks.file_text(data_dir, world, kind, entries).map_err(|e| vec![e])?;
    std::fs::write(&path, text).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    let new: Vec<String> = hooks.check(data_dir).into_iter().filter(|p| !old_problems.contains(p)).collect();
    if !new.is_empty() {
        match before {
            Some(t) => std::fs::write(&path, t).ok(),
            None => std::fs::remove_file(&path).ok(),
        };
        return Err(new);
    }
    Ok(())
}
