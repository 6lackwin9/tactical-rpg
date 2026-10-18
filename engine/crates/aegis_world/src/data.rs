//! The data the game loop reads: the content catalog, the world catalogs (WorldCatalog: lineage, merged entries,
//! resolve with overrides), region map templates (MapCatalog), quest templates (QuestCatalog), service lines
//! (ServiceCatalog) and the XP curve. Loaded from the Godot exports for now (parity/out: catalog.json, worlds.json,
//! game_data.json); the data-independence port reads data/ directly later.

use crate::json::{self, empty, Dict};
use aegis_data::Catalog;
use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

pub const PARTY_MAX: i64 = 4;

#[derive(Debug, Clone, Default)]
pub struct World {
    pub manifest: Dict,
    /// kind -> own entries (normalised, each with `_world` and `_qid`), file order.
    pub own: IndexMap<String, Vec<Dict>>,
}

#[derive(Debug, Clone, Default)]
pub struct XpCurve {
    pub xp_to_next: Vec<i64>,
    pub base: f64,
    pub per_level: f64,
    pub tier_mult: IndexMap<String, f64>,
    pub level_delta_step: f64,
    pub level_delta_min: f64,
    pub level_delta_max: f64,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub cat: Arc<Catalog>,
    /// Raw content entries per kind (ContentCatalog.get_entry).
    pub content: IndexMap<String, IndexMap<String, Dict>>,
    pub worlds: IndexMap<String, World>,
    pub maps: Vec<Dict>,
    pub quest_templates: Vec<Dict>,
    pub services: Vec<Dict>,
    pub xp: XpCurve,
}

fn read(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn dicts(v: &Value) -> Vec<Dict> {
    json::arr(v).iter().filter_map(|x| x.as_object().cloned()).collect()
}

impl Data {
    /// From the Godot export folder (catalog.json, worlds.json, game_data.json).
    pub fn load(dir: &Path) -> Result<Self> {
        let catalog = read(&dir.join("catalog.json"))?;
        let worlds = read(&dir.join("worlds.json"))?;
        let game = read(&dir.join("game_data.json"))?;
        Ok(Self::from_values(&catalog, &worlds, &game))
    }

    pub fn from_values(catalog: &Value, worlds: &Value, game: &Value) -> Self {
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
        let g = json::dict(game);
        let curve = json::dget(g, "xp_curve");
        let kill = json::dget(curve, "kill_xp");
        let xp = XpCurve {
            xp_to_next: json::aget(curve, "xp_to_next").iter().map(json::to_int).collect(),
            base: json::f(kill, "base", 10.0),
            per_level: json::f(kill, "per_level", 5.0),
            tier_mult: json::dget(kill, "tier_mult").iter().map(|(k, v)| (k.clone(), json::to_float_or(v, 1.0))).collect(),
            level_delta_step: json::f(kill, "level_delta_step", 0.15),
            level_delta_min: json::f(kill, "level_delta_min", 0.25),
            level_delta_max: json::f(kill, "level_delta_max", 2.0),
        };
        Self {
            cat: Arc::new(Catalog::from_value(catalog)),
            content,
            worlds: world_map,
            maps: dicts(json::get(g, "maps")),
            quest_templates: dicts(json::get(g, "quests")),
            services: dicts(json::get(g, "services")),
            xp,
        }
    }

    // ---- ContentCatalog ----------------------------------------------------------------------------------------

    /// ContentCatalog.get_entry ({} when missing).
    pub fn entry(&self, kind: &str, id: &str) -> &Dict {
        self.content.get(kind).and_then(|m| m.get(id)).unwrap_or_else(|| empty())
    }

    pub fn has(&self, kind: &str, id: &str) -> bool {
        self.content.get(kind).is_some_and(|m| m.contains_key(id))
    }

    pub fn item_name(&self, id: &str) -> String {
        json::s_or(self.entry("items", id), "name", id)
    }

    // ---- MapCatalog / QuestCatalog / ServiceCatalog ------------------------------------------------------------

    pub fn map(&self, id: &str) -> &Dict {
        if id.is_empty() {
            return empty();
        }
        self.maps.iter().find(|m| json::s(m, "id") == id).unwrap_or_else(|| empty())
    }

    pub fn quest_template(&self, id: &str) -> &Dict {
        if id.is_empty() {
            return empty();
        }
        self.quest_templates.iter().find(|m| json::s(m, "id") == id).unwrap_or_else(|| empty())
    }

    pub fn has_quest_template(&self, id: &str) -> bool {
        !self.quest_template(id).is_empty()
    }

    /// ServiceCatalog.text_of: the base line, else DialogueRules.DEFAULT_LINES.
    pub fn service_text(&self, id: &str) -> String {
        match self.services.iter().find(|m| json::s(m, "id") == id) {
            Some(e) => json::s(e, "text"),
            None => match id {
                "greeting" => "{npc} looks up.",
                "offer" => "About {quest}...",
                "accept" => "We will do it.",
                "decline" => "Not now.",
                "turn_in" => "We finished: {quest}.",
                "shop" => "Show me your wares.",
                "farewell" => "Farewell.",
                _ => "",
            }
            .into(),
        }
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

    pub fn split_qid(r: &str) -> Option<(String, String)> {
        let mut parts = r.splitn(2, ':').filter(|p| !p.is_empty());
        let a = parts.next()?;
        let b = parts.next()?;
        Some((a.to_string(), b.to_string()))
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
            let Some((tw, local)) = Self::split_qid(r) else { return empty() };
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

    /// The nearest manifest in the lineage with `party_max` >= 1, else PARTY_MAX.
    pub fn party_max(&self, w: &str) -> i64 {
        for lw in self.lineage(w).iter().rev() {
            let cap = json::get(self.manifest(lw), "party_max");
            if json::is_int(cap) && json::to_int(cap) >= 1 {
                return json::to_int(cap).min(PARTY_MAX);
            }
        }
        PARTY_MAX
    }

    /// The world's own non-empty `start_party`, else the nearest ancestor's.
    pub fn start_party(&self, w: &str) -> Vec<String> {
        for lw in self.lineage(w).iter().rev() {
            let ids = json::sget(self.manifest(lw), "start_party");
            if !ids.is_empty() {
                return ids;
            }
        }
        Vec::new()
    }

    /// Problems that keep world character `id` from joining the party (RECRUIT1-L 1.2).
    pub fn party_character_errors(&self, w: &str, id: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let c = self.resolve(w, "characters", id);
        if c.is_empty() {
            errors.push(format!("'{id}' is not a known character."));
            return errors;
        }
        let role = json::s_or(c, "role", "npc");
        if role != "party" {
            errors.push(format!("Party hero '{id}' needs role party (has {role})."));
        }
        let base_id = json::s(c, "base_unit");
        let base = self.entry("units", &base_id);
        if base.is_empty() || json::s_or(base, "side", "enemy") != "party" {
            errors.push(format!("Party hero '{id}' must play as a party unit (base unit '{base_id}')."));
        }
        if !json::sget(c, "named_items").is_empty() || !json::sget(c, "disciplines").is_empty() || !json::dget(c, "tuning").is_empty() {
            errors.push(format!("Party hero '{id}' may not carry named items, disciplines or tuning yet (RECRUIT1)."));
        }
        errors
    }

    /// WorldFactory.unit_entry: the base unit's entry wearing the character's name, art, side and background.
    pub fn character_unit_entry(&self, character: &Dict) -> Dict {
        let mut entry = self.entry("units", &json::s(character, "base_unit")).clone();
        if entry.is_empty() {
            return entry;
        }
        let name = json::s_or(character, "name", &json::s(&entry, "name"));
        entry.insert("name".into(), Value::String(name));
        let art = json::s(character, "art");
        if !art.is_empty() {
            entry.insert("art".into(), Value::String(art));
        }
        let role = json::s(character, "role");
        if role == "party" || role == "enemy" {
            entry.insert("side".into(), Value::String(role));
        }
        let desc = json::s_or(character, "background", &json::s(&entry, "description"));
        entry.insert("description".into(), Value::String(desc));
        entry
    }

    /// WorldFactory.character_passives: its own, its faction's, its disciplines' and its named items', once each.
    pub fn character_passives(&self, character: &Dict, w: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut add = |raw: &Value| {
            for id in json::strings(raw) {
                if !out.contains(&id) {
                    out.push(id);
                }
            }
        };
        add(json::get(character, "passives"));
        add(json::get(self.resolve(w, "factions", &json::s(character, "faction")), "grants"));
        for r in json::aget(character, "disciplines") {
            add(json::get(self.resolve(w, "disciplines", &json::gstr(r)), "passives"));
        }
        for r in json::aget(character, "named_items") {
            add(json::get(self.resolve(w, "items", &json::gstr(r)), "grants"));
        }
        out
    }
}
