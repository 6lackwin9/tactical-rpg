//! The content store (ContentCatalog.get_entry) and the world graph (WorldCatalog): worlds in `data/worlds/<id>/`
//! (`world.json` plus one file per kind), inheritance through `extends` (lineage), merged entries, overrides,
//! connections and qualified ids (`<world>:<id>`).

use crate::file::{entries_of, read};
use crate::schema::{fields, normalize, sget};
use aegis_core::json::{self, empty, Dict};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct World {
    pub manifest: Dict,
    /// kind -> own entries (normalised, each with `_world` and `_qid`), file order.
    pub own: IndexMap<String, Vec<Dict>>,
}

/// The base catalog's entries and the worlds, as the loader (or a Godot export) gives them.
#[derive(Debug, Clone, Default)]
pub struct Catalogs {
    /// Raw content entries per kind (ContentCatalog.get_entry).
    pub content: IndexMap<String, IndexMap<String, Dict>>,
    pub worlds: IndexMap<String, World>,
}

fn dicts(v: &Value) -> Vec<Dict> {
    json::arr(v).iter().filter_map(|x| x.as_object().cloned()).collect()
}

/// `<world>:<id>` -> (world, id); None unless both parts are there.
pub fn split_qid(r: &str) -> Option<(String, String)> {
    let mut parts = r.splitn(2, ':').filter(|p| !p.is_empty());
    let a = parts.next()?;
    let b = parts.next()?;
    Some((a.to_string(), b.to_string()))
}

/// data/worlds/<world>/world.json.
pub fn manifest_path(data_dir: &Path, world: &str) -> PathBuf {
    data_dir.join("worlds").join(world).join("world.json")
}

/// data/worlds/<world>/<kind>.json.
pub fn kind_path(data_dir: &Path, world: &str, kind: &str) -> PathBuf {
    data_dir.join("worlds").join(world).join(format!("{kind}.json"))
}

impl Catalogs {
    /// From the loader's values: catalog {kind: [entry]} and worlds {id: {manifest, lineage, own: {kind: [entry]}}}.
    pub fn from_values(catalog: &Value, worlds: &Value) -> Self {
        let mut content: IndexMap<String, IndexMap<String, Dict>> = IndexMap::new();
        for (kind, rows) in json::dict(catalog) {
            let mut by_id = IndexMap::new();
            for e in dicts(rows) {
                by_id.insert(json::s(&e, "id"), e);
            }
            content.insert(kind.clone(), by_id);
        }
        let mut world_map = IndexMap::new();
        for (id, w) in json::dict(worlds) {
            let w = json::dict(w);
            let mut own = IndexMap::new();
            for (kind, rows) in json::dget(w, "own") {
                own.insert(kind.clone(), dicts(rows));
            }
            world_map.insert(id.clone(), World { manifest: json::dget(w, "manifest").clone(), own });
        }
        Self { content, worlds: world_map }
    }

    // ---- ContentCatalog ----------------------------------------------------------------------------------------

    /// ContentCatalog.get_entry ({} when missing).
    pub fn entry(&self, kind: &str, id: &str) -> &Dict {
        self.content.get(kind).and_then(|m| m.get(id)).unwrap_or_else(|| empty())
    }

    pub fn has(&self, kind: &str, id: &str) -> bool {
        self.content.get(kind).is_some_and(|m| m.contains_key(id))
    }

    // ---- WorldCatalog ------------------------------------------------------------------------------------------

    pub fn world_ids(&self) -> Vec<String> {
        let mut out: Vec<String> = self.worlds.keys().cloned().collect();
        out.sort();
        out
    }

    pub fn has_world(&self, w: &str) -> bool {
        self.worlds.contains_key(w)
    }

    pub fn manifest(&self, w: &str) -> &Dict {
        self.worlds.get(w).map(|x| &x.manifest).unwrap_or_else(|| empty())
    }

    /// Load order: ancestors first (parents in `extends` order), the world last.
    pub fn lineage(&self, w: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        self.visit(w, &mut out, &mut stack);
        out
    }

    fn visit(&self, w: &str, out: &mut Vec<String>, stack: &mut Vec<String>) {
        if out.iter().any(|x| x == w) || stack.iter().any(|x| x == w) || !self.has_world(w) {
            return;
        }
        stack.push(w.to_string());
        for parent in json::sget(self.manifest(w), "extends") {
            self.visit(&parent, out, stack);
        }
        stack.pop();
        out.push(w.to_string());
    }

    pub fn own_entries(&self, w: &str, kind: &str) -> &[Dict] {
        self.worlds.get(w).and_then(|x| x.own.get(kind)).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Ancestors' entries then the world's own, minus entries an override replaced.
    pub fn merged_entries(&self, w: &str, kind: &str) -> Vec<&Dict> {
        let mut all = Vec::new();
        let mut overridden: Vec<String> = Vec::new();
        for lw in self.lineage(w) {
            for e in self.own_entries(&lw, kind) {
                all.push(e);
                let ov = json::s(e, "override");
                if !ov.is_empty() {
                    overridden.push(ov);
                }
            }
        }
        all.into_iter().filter(|e| !overridden.contains(&json::s(e, "_qid"))).collect()
    }

    fn own_by_id(&self, w: &str, kind: &str, local: &str) -> Option<&Dict> {
        self.own_entries(w, kind).iter().find(|e| json::s(e, "id") == local)
    }

    /// The entry `r` names as seen from `w` ({} if none): unqualified refs resolve in the world then up the lineage;
    /// overrides are followed (not for a connected world's entry).
    pub fn resolve(&self, w: &str, kind: &str, r: &str) -> &Dict {
        if r.is_empty() {
            return empty();
        }
        let chain = self.lineage(w);
        let found = if r.contains(':') {
            let Some((tw, local)) = split_qid(r) else { return empty() };
            self.own_by_id(&tw, kind, &local)
        } else {
            chain.iter().rev().find_map(|lw| self.own_by_id(lw, kind, r))
        };
        let Some(mut found) = found else { return empty() };
        if !chain.contains(&json::s(found, "_world")) {
            return found;
        }
        let mut redirects: IndexMap<String, &Dict> = IndexMap::new();
        for lw in &chain {
            for e in self.own_entries(lw, kind) {
                let ov = json::s(e, "override");
                if !ov.is_empty() {
                    redirects.insert(ov, e);
                }
            }
        }
        let mut guard = 0;
        while let Some(next) = redirects.get(&json::s(found, "_qid")) {
            if guard >= 16 {
                break;
            }
            found = next;
            guard += 1;
        }
        found
    }

    pub fn has_entry(&self, w: &str, kind: &str, r: &str) -> bool {
        !self.resolve(w, kind, r).is_empty()
    }

    /// Is `other` visible from `w` (itself, an ancestor, or connected; connections are inherited)?
    pub fn can_reach_world(&self, w: &str, other: &str) -> bool {
        let chain = self.lineage(w);
        if chain.iter().any(|x| x == other) {
            return true;
        }
        chain.iter().any(|lw| json::sget(self.manifest(lw), "connects").iter().any(|c| c == other))
    }
}

/// WorldCatalog.reload_from_disk: every world folder with a world.json, its manifest and its own entries of `kinds`
/// normalized by the schema's "world" section, as {id: {manifest, lineage, own: {kind: [entry]}}}.
pub fn load(data_dir: &Path, schema: &Value, kinds: &[&str], errors: &mut Vec<String>) -> Value {
    let root = data_dir.join("worlds");
    let mut ids: Vec<String> = std::fs::read_dir(&root)
        .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    ids.retain(|id| !id.starts_with('.') && manifest_path(data_dir, id).is_file());
    ids.sort();
    let mut manifests: Vec<(String, Dict)> = Vec::new();
    let mut owns: Vec<Dict> = Vec::new();
    for id in &ids {
        let header = read(&manifest_path(data_dir, id), errors).and_then(|v| v.as_object().cloned()).unwrap_or_default();
        let mut header = normalize(fields(schema, "world", "world"), &header, true);
        header.insert("id".into(), Value::from(id.clone()));
        let mut own = Dict::new();
        for kind in kinds {
            let rows: Vec<Value> = match read(&kind_path(data_dir, id, kind), errors) {
                Some(doc) => entries_of(doc).0,
                None => Vec::new(),
            };
            let mut list = Vec::new();
            for r in rows.iter().filter_map(Value::as_object) {
                let mut e = normalize(fields(schema, "world", kind), r, true);
                let local = sget(&e, "id", "");
                e.insert("_world".into(), Value::from(id.clone()));
                e.insert("_qid".into(), Value::from(format!("{id}:{local}")));
                list.push(Value::Object(e));
            }
            own.insert(kind.to_string(), Value::Array(list));
        }
        manifests.push((id.clone(), header));
        owns.push(own);
    }
    let mut out = Dict::new();
    for (i, (id, manifest)) in manifests.iter().enumerate() {
        let mut lineage = Vec::new();
        visit(id, &manifests, &mut lineage, &mut Vec::new());
        let mut w = Dict::new();
        w.insert("manifest".into(), Value::Object(manifest.clone()));
        w.insert("lineage".into(), json::strs(&lineage));
        w.insert("own".into(), Value::Object(owns[i].clone()));
        out.insert(id.clone(), Value::Object(w));
    }
    Value::Object(out)
}

/// WorldCatalog._visit: ancestors first (parents in `extends` order), the world last.
fn visit(id: &str, manifests: &[(String, Dict)], out: &mut Vec<String>, stack: &mut Vec<String>) {
    let Some((_, m)) = manifests.iter().find(|(w, _)| w == id) else { return };
    if out.iter().any(|x| x == id) || stack.iter().any(|x| x == id) {
        return;
    }
    stack.push(id.to_string());
    for parent in json::strings(json::get(m, "extends")) {
        visit(&parent, manifests, out, stack);
    }
    stack.pop();
    out.push(id.to_string());
}
