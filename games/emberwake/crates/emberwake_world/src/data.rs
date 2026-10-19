//! The data the game loop reads: the engine's content store and world graph (`aegis_content::Catalogs`:
//! ContentCatalog entries, WorldCatalog lineage, merged entries, resolve with overrides; `Data` derefs to it), the
//! typed catalog, region map templates (MapCatalog), quest templates (QuestCatalog), service lines (ServiceCatalog)
//! and the XP curve, plus the game's world rules (party size, start party, world characters). Read from the
//! project's data/ folder by the game's loader (`source`), or from the Godot exports (parity/out: catalog.json,
//! worlds.json, game_data.json) for parity checks.

use aegis_content::Catalogs;
use aegis_core::json::{self, empty, Dict};
use emberwake_data::Catalog;
use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

pub const PARTY_MAX: i64 = 4;

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
    /// Raw content entries per kind (item families included) and the worlds.
    pub catalogs: Catalogs,
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

impl std::ops::Deref for Data {
    type Target = Catalogs;

    fn deref(&self) -> &Catalogs {
        &self.catalogs
    }
}

impl Data {
    /// A data folder read by the engine's own loader (`source`), or a Godot export folder (has catalog.json):
    /// the data and the load problems (bad files or entries are skipped and reported, like Godot's load_errors).
    pub fn open(dir: &Path) -> Result<(Self, Vec<String>)> {
        if dir.join("catalog.json").is_file() {
            return Ok((Self::load(dir)?, Vec::new()));
        }
        anyhow::ensure!(dir.join("catalog").is_dir(), "{} is neither a data folder (catalog/, worlds/, ...) nor a Godot export folder", dir.display());
        let src = emberwake_content::source::load(dir);
        Ok((Self::from_values(&src.catalog, &src.worlds, &src.game), src.errors))
    }

    /// The game's data/ folder (games/emberwake/data) found from `start` (the game folder, the repository, or a folder
    /// inside either).
    pub fn find_data_dir(start: &Path) -> Option<std::path::PathBuf> {
        aegis_content::manifest::find_game_dir(start, crate::GAME_ID).map(|g| g.join("data")).filter(|d| d.join("catalog").is_dir() && d.join("worlds").is_dir())
    }

    /// From the Godot export folder (catalog.json, worlds.json, game_data.json).
    pub fn load(dir: &Path) -> Result<Self> {
        let catalog = read(&dir.join("catalog.json"))?;
        let worlds = read(&dir.join("worlds.json"))?;
        let game = read(&dir.join("game_data.json"))?;
        Ok(Self::from_values(&catalog, &worlds, &game))
    }

    pub fn from_values(catalog: &Value, worlds: &Value, game: &Value) -> Self {
        let mut catalogs = Catalogs::from_values(catalog, worlds);
        let g = json::dict(game);
        let mut families = IndexMap::new();
        for e in dicts(json::get(g, "item_families")) {
            families.insert(json::s(&e, "id"), e);
        }
        catalogs.content.insert("item_families".into(), families);
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
            catalogs,
            maps: dicts(json::get(g, "maps")),
            quest_templates: dicts(json::get(g, "quests")),
            services: dicts(json::get(g, "services")),
            xp,
        }
    }

    pub fn item_name(&self, id: &str) -> String {
        json::s_or(self.entry("items", id), "name", id)
    }

    /// The item's description with its tokens filled from the resolved row (ItemText.render, as ContentFactory.item).
    pub fn item_description(&self, id: &str) -> String {
        let row = self.entry("items", id);
        emberwake_content::source::render_item(&json::s(row, "description"), row)
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

    // ---- WorldCatalog (game rules) ------------------------------------------------------------------------------

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
