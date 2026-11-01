//! The game's content writer on the engine content layer (`aegis_content`: file format, schema storage and checks,
//! upsert / remove / commit): what Godot's ContentCatalog.upsert / save and WorldCatalog.upsert / save_world did, so
//! data/ is edited through a tool (never by hand) without Godot. Entries are given in their stored shape (the shape
//! the data files hold: ranked skill entries, items as family + modifiers) and written in Godot's storage form
//! (ContentSchema.to_storage, SkillRanks.collapse, ItemModifiers.stored, WorldSchema.to_storage) and file format, so a
//! file the writer rewrites without changes stays byte-identical (tests/store.rs). This module is the game's side:
//! its kinds and files, the storage rules of skills, items and game-data files, and the check (every hard problem the
//! content checks find, `check`); a write that adds a problem is put back as it was.

use crate::source::{self, content, expand, fields, resolve_item, RANK_KEYS};
use aegis_content::file::{self, raw_entries, read, stringify};
use aegis_content::schema::{copy, field_errors, has_field, normalize, sget, to_storage};
use aegis_content::store::Hooks;
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub use aegis_content::store::Outcome;

// ---- Storage forms -----------------------------------------------------------------------------------------------

/// ContentSchema.to_storage of a normalized row: hidden and internal fields, `system` false and `origin` system
/// left out; unknown keys kept.
fn content_storage(kind: &str, full: &Dict) -> Dict {
    to_storage(source::schema(), "content", kind, full, |key, v| (key == "system" && !json::to_bool(v)) || (key == "origin" && gstr(v) == "system"))
}

/// CLN-45: a ruleset row stores a switch it leaves to the rules (`ruleset::BY_RULES_FIELDS` at "") as no key (the
/// same value when read), so a row written before the switches were fields re-saves byte for byte.
fn ruleset_storage(full: &Dict) -> Dict {
    let by_rules = |key: &str, v: &Value| rpgkit_rules::ruleset::BY_RULES_FIELDS.contains(&key) && gstr(v).is_empty();
    to_storage(source::schema(), "content", source::RULESET_KIND, full, |key, v| (key == "system" && !json::to_bool(v)) || (key == "origin" && gstr(v) == "system") || by_rules(key, v))
}

/// PORTRAITFX1 5.1: a portrait cue row (and PORTRAITFX1-EX: an expression-effect row) stores its id and only the
/// fields that differ from their schema defaults (a float default compared as normalize snaps it too, so 0.85 is the
/// default whichever way it was read).
fn cue_storage(kind: &str, full: &Dict) -> Dict {
    let defaults: Dict = fields("content", kind).iter().map(json::dict).map(|f| (json::s(f, "key"), json::get(f, "default").clone())).collect();
    let snapped = content(kind, &Dict::new());
    let is_default = |key: &str, v: &Value| defaults.get(key).is_some_and(|d| json::deep_equal(d, v)) || (defaults.contains_key(key) && snapped.get(key).is_some_and(|d| json::deep_equal(d, v)));
    to_storage(source::schema(), "content", kind, full, |key, v| key != "id" && is_default(key, v))
}

/// PORTRAITFX1 7: a dialogue node's and a vignette card's `expression` is stored only when it is not "".
fn without_empty_expressions(kind: &str, entry: &Dict) -> Dict {
    let mut out = entry.clone();
    let list = match kind {
        "dialogues" => "nodes",
        "vignettes" => "cards",
        _ => return out,
    };
    if let Some(Value::Array(items)) = out.get_mut(list) {
        for item in items.iter_mut().filter_map(Value::as_object_mut) {
            if item.get("expression").is_some_and(|x| x.as_str() == Some("")) {
                item.shift_remove("expression");
            }
        }
    }
    out
}

/// INPUT1 FL1: a unit's `deck.follow_ups` stores only its non-empty lists, and not at all when none is left (so a
/// deck without lists writes as before).
fn without_empty_follow_ups(kind: &str, entry: &Dict) -> Dict {
    let mut out = entry.clone();
    if kind != "units" {
        return out;
    }
    if let Some(Value::Object(deck)) = out.get_mut("deck") {
        let empty = match deck.get_mut("follow_ups") {
            None => false,
            Some(Value::Object(lists)) => {
                lists.retain(|_, l| !l.as_array().is_some_and(Vec::is_empty));
                lists.is_empty()
            }
            Some(_) => false,
        };
        if empty {
            deck.shift_remove("follow_ups");
        }
    }
    out
}

/// WorldSchema.to_storage of a raw entry (normalized first; an empty `override` is left out).
pub fn world_storage(kind: &str, entry: &Dict) -> Dict {
    aegis_content::schema::world_storage(source::schema(), kind, entry)
}

/// Resolved item keys the stored form leaves out (ItemModifiers.stored; RULESET1 K1 adds the family's reach rule).
const ITEM_DERIVED: [&str; 14] = ["item_type", "weapon_type", "two_handed", "off_hand_category", "base_power", "base_range", "block_cost", "burden", "mitigation", "required_stats", "stat_bonuses", "effects", "threshold_tiers", rpgkit_data::ITEM_REACH_RULE];

/// SkillKinds.has_field: common fields, the ATTACK-only stance override, and the fields of the kind's sections (one
/// table with the checks; RULESET1-S added the timeline fields there).
fn skill_has_field(kind: &str, key: &str) -> bool {
    crate::check::skill::has_field(kind, key)
}

/// SkillRanks.fold of one entry's rank rows: the ranked entry Godot writes (core keys always, other fields of the
/// kind only when they differ from the default, rank modifiers with the rank fields that differ from rank 1).
fn skill_storage(entry: &Dict) -> Dict {
    const CORE: [&str; 6] = ["id", "name", "kind", "level", "blocks", "description"];
    let rows = expand(entry);
    let first = &rows[0];
    let head = sget(first, "id", "");
    let kind = sget(first, "kind", "");
    let template = sget(first, "template", "");
    let base_template = if template.is_empty() { sget(first, "description", "").replace('{', "{{").replace('}', "}}") } else { template };
    let schema = fields("content", "skills");
    let rank_fields: Vec<String> = schema.iter().map(|f| json::s(json::dict(f), "key")).filter(|k| RANK_KEYS.contains(&k.as_str()) && skill_has_field(&kind, k)).collect();
    let mut mods = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(1) {
        let mut m = Dict::new();
        for k in &rank_fields {
            if !json::deep_equal(json::get(row, k), json::get(first, k)) {
                m.insert(k.clone(), json::get(row, k).clone());
            }
        }
        if sget(row, "description", "") != source::render(&base_template, &kind, row, i + 1) {
            m.insert("description".into(), Value::from(sget(row, "description", "")));
        }
        mods.push(Value::Object(m));
    }
    let mut out = Dict::new();
    for f in schema {
        let f = json::dict(f);
        let key = json::s(f, "key");
        if matches!(key.as_str(), "family" | "tier" | "template") || !skill_has_field(&kind, &key) {
            continue;
        }
        let value = match key.as_str() {
            "id" => Value::from(head.clone()),
            "description" => Value::from(base_template.clone()),
            _ => json::get(first, &key).clone(),
        };
        if !CORE.contains(&key.as_str()) && json::deep_equal(&value, json::get(f, "default")) {
            continue;
        }
        out.insert(key, value);
    }
    for (k, v) in first {
        if !out.contains_key(k) && !has_field(source::schema(), "content", "skills", k) {
            out.insert(k.clone(), v.clone());
        }
    }
    if !mods.is_empty() {
        out.insert("ranks".into(), Value::Array(mods));
    }
    out
}

/// The stored form of a raw (file-shape) catalog entry of `kind`; `families` are the derived family rows.
pub fn catalog_storage(kind: &str, entry: &Dict, families: &[Dict]) -> Result<Dict, String> {
    match kind {
        "skills" => Ok(skill_storage(entry)),
        "items" => {
            let fam_id = sget(entry, "family", "");
            let family = families.iter().find(|f| sget(f, "id", "") == fam_id).ok_or_else(|| format!("item '{}' has no known family ('{fam_id}'); a version 2 file stores family + modifiers only", sget(entry, "id", "")))?;
            let row = content_storage("items", &content("items", &resolve_item(entry, family)));
            // ItemModifiers.stored
            let has_hand = !sget(&row, "hand", "").is_empty();
            Ok(row.into_iter().filter(|(k, _)| !(ITEM_DERIVED.contains(&k.as_str()) || (k == "slot" && has_hand) || (k == "hand" && !has_hand))).map(|(k, v)| (k, copy(&v))).collect())
        }
        "item_families" => Ok(content_storage(kind, &source::derive_family(&content(kind, entry)))),
        k if k == source::PORTRAIT_KIND || k == source::EXPRESSION_FX_KIND => Ok(cue_storage(kind, &content(kind, entry))),
        k if k == source::RULESET_KIND => Ok(ruleset_storage(&content(kind, entry))),
        _ => Ok(content_storage(kind, &content(kind, entry))),
    }
}

// ---- Files -------------------------------------------------------------------------------------------------------

/// Where an entry of `kind` lives: data/catalog/<kind>.json, or data/worlds/<world>/<kind>.json.
pub fn file_of(data_dir: &Path, world: Option<&str>, kind: &str) -> PathBuf {
    match (world, game_kind(kind)) {
        (Some(w), _) => aegis_content::world::kind_path(data_dir, w, kind),
        (None, Some((_, file, _, _))) => data_dir.join(file),
        (None, None) => data_dir.join("catalog").join(format!("{kind}.json")),
    }
}

/// The game-data files beside the catalogs (MapCatalog, QuestCatalog, ServiceCatalog): kind, file, the file's
/// "kind", the stored fields (to_storage keeps exactly these) and the normalize.
type GameKind = (&'static str, &'static str, &'static str, &'static [&'static str]);
pub const GAME_KINDS: [GameKind; 3] = [
    ("maps", "maps/maps.json", "map_templates", &["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "props", "tags"]),
    ("quests", "quests/quests.json", "quest_templates", &["id", "name", "description", "objectives", "rewards", "tags"]),
    ("services", "dialogue/services.json", "dialogue_services", &["id", "text"]),
];

/// The kinds an upsert takes in the base catalog (catalog kinds, item families, the derived stats (DERIVE1), the
/// rulesets (RULESET1), the portrait cues (PORTRAITFX1), the expression effects (PORTRAITFX1-EX), the combat hook
/// kinds (HOOKS1), the terms (TERMS1), game-data kinds).
const BASE_KINDS: [&str; 19] = [
    "classes", "units", "skills", "items", "item_families", "passives", "encounters", "terrain", source::DERIVED_KIND, source::RULESET_KIND, source::PORTRAIT_KIND, source::EXPRESSION_FX_KIND, source::HOOK_KINDS[0], source::HOOK_KINDS[1], source::HOOK_KINDS[2], source::TERMS_KIND, "maps", "quests", "services",
];

fn game_kind(kind: &str) -> Option<GameKind> {
    GAME_KINDS.iter().copied().find(|g| g.0 == kind)
}

/// MapCatalog / QuestCatalog / ServiceCatalog.to_storage: the normalized entry's stored fields, in that order.
fn game_storage(kind: &str, entry: &Dict) -> Dict {
    let full = match kind {
        "maps" => source::map_template(entry),
        "quests" => source::quest_template(entry),
        _ => source::service(entry),
    };
    game_kind(kind).map(|g| g.3).unwrap_or(&[]).iter().map(|k| (k.to_string(), json::get(&full, k).clone())).collect()
}

fn derived_families(data_dir: &Path) -> Vec<Dict> {
    let mut errors = Vec::new();
    let doc = read(&data_dir.join("catalog/item_families.json"), &mut errors).map(|d| file::entries_of(d).0).unwrap_or_default();
    doc.iter().filter_map(Value::as_object).map(|e| source::derive_family(&content("item_families", e))).collect()
}

/// ContentCatalog.version_of / WorldCatalog.VERSION.
fn version_of(world: Option<&str>, kind: &str) -> i64 {
    match (world, kind) {
        (None, "skills" | "items") => 2,
        _ => 1,
    }
}

/// Every entry of the file in storage form, as the save would write it: the document text.
fn rewrite(data_dir: &Path, world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<String, String> {
    let game = if world.is_none() { game_kind(kind) } else { None };
    let families = derived_families(data_dir);
    let mut stored = Vec::new();
    for e in entries {
        // PHASEFOLD 1.4: an encounter written before PHASEFOLD is stored with its wave as an arrival phase.
        let e = &source::folded(kind, e);
        stored.push(Value::Object(match (world, game) {
            (Some(_), _) => world_storage(kind, &without_empty_follow_ups(kind, &without_empty_expressions(kind, e))),
            (None, Some(_)) => game_storage(kind, e),
            (None, None) => catalog_storage(kind, &without_empty_follow_ups(kind, e), &families)?,
        }));
    }
    Ok(file::document(game.map(|g| g.2).unwrap_or(kind), version_of(world, kind), stored))
}

/// The file text a save of `kind` would write now (tests: equal to the file when nothing changed).
pub fn resave_text(data_dir: &Path, world: Option<&str>, kind: &str) -> Result<String, String> {
    rewrite(data_dir, world, kind, &raw_entries(&file_of(data_dir, world, kind)))
}

/// The stored manifest of `world` (raw, as the file holds it), None when the world has no world.json.
fn stored_manifest(data_dir: &Path, world: &str) -> Result<Option<Dict>, String> {
    let path = aegis_content::world::manifest_path(data_dir, world);
    if !path.is_file() {
        return Ok(None);
    }
    let mut errors = Vec::new();
    read(&path, &mut errors).and_then(|v| v.as_object().cloned()).map(Some).ok_or_else(|| format!("{}: {}", path.display(), errors.pop().unwrap_or_else(|| "not an object".into())))
}

/// WorldCatalog.save_world's document of a raw manifest: normalized against `world.world`, its id set, every field in
/// schema order (unknown keys after), version 1.
fn manifest_doc(world: &str, raw: &Dict) -> Dict {
    let mut m = normalize(fields("world", "world"), raw, true);
    m.insert("id".into(), Value::from(world));
    let mut doc = world_storage("world", &m);
    doc.insert("version".into(), json::int(1));
    doc
}

/// The world manifest (world.json) as WorldCatalog.save_world writes it.
pub fn manifest_text(data_dir: &Path, world: &str) -> Result<String, String> {
    let path = aegis_content::world::manifest_path(data_dir, world);
    let raw = stored_manifest(data_dir, world)?.ok_or_else(|| format!("{}: missing", path.display()))?;
    Ok(stringify(&Value::Object(manifest_doc(world, &raw))))
}

/// WORLDW1: the manifest of `world` after a write of `patch` (world.world fields only; `id`, when given, is the
/// world's): merged over the stored manifest, or over the schema defaults for a new world; refused when a value has
/// the wrong type for its field or the result fails the schema checks (id, name, enum options such as `ruleset`).
/// Rendered as `manifest_text` renders; unless `full`, a field the stored file leaves out stays out while it holds its
/// default and the patch does not name it, so a resave of an untouched manifest is byte-identical.
pub fn world_manifest_text(data_dir: &Path, world: &str, patch: &Dict, full: bool) -> Result<String, Vec<String>> {
    let schema = fields("world", "world");
    let patch = json::normalize_dict(patch);
    let mut errors: Vec<String> = patch
        .keys()
        .filter(|k| !has_field(source::schema(), "world", "world", k))
        .map(|k| format!("A world manifest has no field '{k}' (fields: {}).", schema.iter().map(|f| json::s(json::dict(f), "key")).collect::<Vec<_>>().join(", ")))
        .collect();
    if let Some(id) = patch.get("id").filter(|id| gstr(id) != world) {
        errors.push(format!("ID {id} is not the world's id '{world}' (a world is renamed by a new folder)."));
    }
    errors.extend(aegis_content::schema::type_errors(schema, &patch));
    if !errors.is_empty() {
        return Err(errors);
    }
    let stored = stored_manifest(data_dir, world).map_err(|e| vec![e])?;
    let mut raw = stored.clone().unwrap_or_default();
    for (k, v) in &patch {
        raw.insert(k.clone(), v.clone());
    }
    let mut m = normalize(schema, &raw, true);
    m.insert("id".into(), Value::from(world));
    let errors = field_errors(source::schema(), "world", "world", &m);
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut doc = manifest_doc(world, &raw);
    if let (false, Some(stored)) = (full, &stored) {
        for f in schema {
            let f = json::dict(f);
            let key = json::s(f, "key");
            if !stored.contains_key(&key) && !patch.contains_key(&key) && doc.get(&key).is_some_and(|v| json::deep_equal(v, json::get(f, "default"))) {
                doc.shift_remove(&key);
            }
        }
    }
    Ok(stringify(&Value::Object(doc)))
}

/// WORLDW1: creates or updates world `world`'s manifest with `patch` (`world_manifest_text`), kept only when the data
/// folder adds no blocking problem and no world manifest problem (`manifest_blocking`).
pub fn set_world(data_dir: &Path, world: &str, patch: &Dict, full: bool) -> Result<Outcome, Vec<String>> {
    aegis_content::store::set_world(&RpgContent, data_dir, world, patch, full)
}

// ---- Upsert / remove / check -------------------------------------------------------------------------------------

/// The RPG content hooks for the engine writer (the kit's kinds, schema checks, storage rules and folder checks).
pub struct RpgContent;

impl Hooks for RpgContent {
    fn kinds(&self, world: Option<&str>) -> &'static [&'static str] {
        if world.is_some() {
            &source::WORLD_KINDS
        } else {
            &BASE_KINDS
        }
    }

    fn file_of(&self, data_dir: &Path, world: Option<&str>, kind: &str) -> PathBuf {
        file_of(data_dir, world, kind)
    }

    /// Schema checks of the normalized entry (ContentSchema.field_errors / WorldSchema shape): id, name, enum
    /// options; game-data kinds check the id only.
    fn validate(&self, world: Option<&str>, kind: &str, entry: &Dict) -> Result<Dict, Vec<String>> {
        let section = if world.is_some() { "world" } else { "content" };
        let game = world.is_none() && game_kind(kind).is_some();
        let entry = &source::folded(kind, entry);
        let full = if world.is_some() { normalize(fields("world", kind), entry, true) } else if game { game_storage(kind, entry) } else { content(kind, entry) };
        if world.is_none() && kind == source::PORTRAIT_KIND && sget(&full, "trigger", "") == rpgkit_data::PORTRAIT_METER_TRIGGER {
            // PORTRAITFX1 5.1: the reserved trigger is refused with its reason, not as an unknown option
            return Err(vec![crate::check::portrait::METER_MESSAGE.into()]);
        }
        let errors = field_errors(source::schema(), if game { "none" } else { section }, kind, &full);
        if errors.is_empty() {
            Ok(full)
        } else {
            Err(errors)
        }
    }

    fn file_text(&self, data_dir: &Path, world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<String, String> {
        rewrite(data_dir, world, kind, entries)
    }

    fn check(&self, data_dir: &Path) -> Vec<String> {
        check(data_dir)
    }

    fn blocking(&self, data_dir: &Path) -> Vec<String> {
        blocking(data_dir)
    }

    fn warnings(&self, data_dir: &Path) -> Vec<String> {
        warnings(data_dir)
    }

    fn world_manifest_text(&self, data_dir: &Path, world: &str, patch: &Dict, full: bool) -> Result<String, Vec<String>> {
        world_manifest_text(data_dir, world, patch, full)
    }

    fn world_blocking(&self, data_dir: &Path) -> Vec<String> {
        manifest_blocking(data_dir)
    }
}

/// ContentCatalog.upsert / WorldCatalog.upsert: validates `entry` (stored shape), replaces the entry with its id (or
/// `replace_id`) or appends it, writes the file, and keeps it only when the whole data folder still loads cleanly.
pub fn upsert(data_dir: &Path, world: Option<&str>, kind: &str, entry: &Dict, replace_id: Option<&str>) -> Result<Outcome, Vec<String>> {
    aegis_content::store::upsert(&RpgContent, data_dir, world, kind, entry, replace_id)
}

/// Removes the entry `id` of `kind` (no reference check beyond the data folder still loading cleanly).
pub fn remove(data_dir: &Path, world: Option<&str>, kind: &str, id: &str) -> Result<Outcome, Vec<String>> {
    aegis_content::store::remove(&RpgContent, data_dir, world, kind, id)
}

/// The checks view of a data folder: the loaded catalog and worlds, the art keys of its asset manifest, whether it has
/// a terrain file, and the load problems.
pub fn view(data_dir: &Path) -> crate::check::View {
    view_of(data_dir, &source::load(data_dir))
}

fn view_of(data_dir: &Path, src: &source::Source) -> crate::check::View {
    let mut view = crate::check::View::new(&src.catalog, &src.worlds, &src.game, data_dir.parent().map(aegis_core::assets::Layers::for_game).unwrap_or_default());
    view.terrain_file = data_dir.join("catalog/terrain.json").is_file();
    view.load_errors = src.errors.clone();
    view
}

fn lines(report: crate::check::Report) -> impl Iterator<Item = String> {
    report.into_iter().flat_map(|(key, found)| found.into_iter().map(move |m| format!("{key}: {m}")))
}

/// Every hard problem of the data folder, one "<key>: <message>" line each: the catalog (ContentCatalog.validate_all,
/// load problems under "load"), the worlds (WorldCatalog.validate_all) and the game manifest.
pub fn check(data_dir: &Path) -> Vec<String> {
    let src = source::load(data_dir);
    let view = view_of(data_dir, &src);
    let mut out: Vec<String> = lines(crate::check::validate_all(&view)).chain(lines(crate::check::world::validate_all(&view))).collect();
    out.extend(manifest_problems(data_dir, &src.worlds));
    out
}

/// The problems that refuse a write: the catalog's (ContentCatalog.upsert validates the entry), the world entries'
/// blocking ones only (WorldCatalog.upsert: unresolved references do not block) and the game manifest's.
pub fn blocking(data_dir: &Path) -> Vec<String> {
    let src = source::load(data_dir);
    let view = view_of(data_dir, &src);
    let mut out: Vec<String> = lines(crate::check::validate_all(&view)).collect();
    for w in view.worlds.world_ids() {
        for kind in crate::check::world::KINDS {
            for e in view.worlds.own_entries(&w, kind) {
                let id = json::s(e, "id");
                out.extend(crate::check::world::blocking_errors(&view, &w, kind, e, &id).into_iter().map(|m| format!("{w}/{kind}:{id}: {m}")));
            }
        }
    }
    out.extend(manifest_problems(data_dir, &src.worlds));
    out
}

/// The problems a world manifest write may not add (WORLDW1; WorldCatalog.save_world checks the manifest): the
/// `blocking` ones (load problems included) and every world's manifest problems (WorldCatalog.manifest_errors, as
/// "<world>/world: <message>": a manifest change can break its children, e.g. a circular `extends`).
pub fn manifest_blocking(data_dir: &Path) -> Vec<String> {
    let mut out = blocking(data_dir);
    let view = view(data_dir);
    for w in view.worlds.world_ids() {
        out.extend(crate::check::world::manifest_errors(&view, &w).into_iter().map(|m| format!("{w}/world: {m}")));
    }
    out
}

/// Every soft problem of the data folder, one line each (ContentCatalog.warnings_all as "<key>: <message>",
/// WorldCatalog.warnings_all, whose lines name their entry): never blocking.
pub fn warnings(data_dir: &Path) -> Vec<String> {
    let view = view(data_dir);
    let mut out: Vec<String> = lines(crate::check::warnings_all(&view)).collect();
    out.extend(crate::check::world::warnings_all(&view).into_values().flatten());
    out
}

/// The manifest beside the data folder (the game's game.json), when there is one: its own problems, and its
/// worlds against the world folders (the list is metadata; the loader reads the folders).
fn manifest_problems(data_dir: &Path, worlds: &Value) -> Vec<String> {
    let Some(path) = data_dir.parent().map(|g| g.join(aegis_content::manifest::FILE)).filter(|p| p.is_file()) else { return Vec::new() };
    let m = match aegis_content::manifest::Manifest::read(&path) {
        Ok(m) => m,
        Err(e) => return vec![e],
    };
    let mut out = m.problems();
    let folders: Vec<String> = worlds.as_object().map(|w| w.keys().cloned().collect()).unwrap_or_default();
    for w in m.worlds.iter().filter(|w| !folders.contains(w)) {
        out.push(format!("game.json: world '{w}' has no folder in data/worlds/."));
    }
    for w in folders.iter().filter(|w| !m.worlds.contains(w)) {
        out.push(format!("game.json: world folder '{w}' is not one of its worlds."));
    }
    out
}
