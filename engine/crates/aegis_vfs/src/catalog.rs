use crate::vfs::{Vfs, VfsError};
use aegis_core::Variant;
use serde_json::Value;
use std::collections::HashMap;

/// Manages content catalogs (skills, items, classes, encounters, terrain, passives).
/// Equivalent to Godot's `ContentCatalog`.
#[derive(Debug, Clone)]
pub struct CatalogStore {
    vfs: Vfs,
    catalogs: HashMap<String, HashMap<String, Variant>>,
}

impl CatalogStore {
    pub fn new(vfs: Vfs) -> Self {
        Self {
            vfs,
            catalogs: HashMap::new(),
        }
    }

    /// Loads a catalog file from `res://data/catalog/{kind}.json`
    pub fn load_catalog(&mut self, kind: &str) -> Result<usize, VfsError> {
        let vpath = format!("res://data/catalog/{}.json", kind);
        if !self.vfs.file_exists(&vpath) {
            return Ok(0);
        }

        let json = self.vfs.read_json(&vpath)?;
        let mut entries = HashMap::new();

        if let Value::Array(list) = json {
            for item in list {
                let var = Variant::from_json(&item);
                let id = var.get_key("id").as_str().to_string();
                if !id.is_empty() {
                    entries.insert(id, var);
                }
            }
        } else if let Value::Object(obj) = json {
            // Some catalogs may be formatted as {"entries": [...]}
            if let Some(Value::Array(list)) = obj.get("entries") {
                for item in list {
                    let var = Variant::from_json(item);
                    let id = var.get_key("id").as_str().to_string();
                    if !id.is_empty() {
                        entries.insert(id, var);
                    }
                }
            }
        }

        let count = entries.len();
        self.catalogs.insert(kind.to_string(), entries);
        Ok(count)
    }

    /// Loads all standard catalogs from the reference game.
    pub fn load_all_standard(&mut self) -> Result<HashMap<String, usize>, VfsError> {
        let standard_kinds = ["classes", "skills", "items", "encounters", "passives", "terrain", "units"];
        let mut results = HashMap::new();
        for kind in standard_kinds {
            let count = self.load_catalog(kind)?;
            results.insert(kind.to_string(), count);
        }
        Ok(results)
    }

    /// Gets an entry by catalog kind and ID.
    pub fn get_entry(&self, kind: &str, id: &str) -> Option<&Variant> {
        self.catalogs.get(kind).and_then(|cat| cat.get(id))
    }

    /// Upserts an entry in memory.
    pub fn upsert_entry(&mut self, kind: &str, id: &str, entry: Variant) {
        self.catalogs
            .entry(kind.to_string())
            .or_default()
            .insert(id.to_string(), entry);
    }

    /// Returns all entries for a catalog kind.
    pub fn get_all(&self, kind: &str) -> Option<&HashMap<String, Variant>> {
        self.catalogs.get(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_in_memory() {
        let vfs = Vfs::new(".", ".");
        let mut store = CatalogStore::new(vfs);

        let mut skill_data = HashMap::new();
        skill_data.insert("id".to_string(), Variant::String("slash".into()));
        skill_data.insert("power".to_string(), Variant::Int(12));

        store.upsert_entry("skills", "slash", Variant::Dictionary(skill_data));

        let retrieved = store.get_entry("skills", "slash").unwrap();
        assert_eq!(retrieved.get_key("power").to_int(), 12);
    }
}
