//! The engine's content writer: what Godot's ContentCatalog.upsert / save and WorldCatalog.upsert / save_world did,
//! so data/ is edited through a tool (never by hand) without Godot. Entries are given in their stored shape (the shape
//! the data files hold: ranked skill entries, items as family + modifiers) and written in Godot's storage form
//! (ContentSchema.to_storage, SkillRanks.collapse, ItemModifiers.stored, WorldSchema.to_storage) and file format
//! (JSON.stringify with tab indent), so a file the writer rewrites without changes stays byte-identical
//! (tests/store.rs). Validation: ids, names, enum options and references by the schema, skill ranks, item families,
//! and the whole data folder must still load without problems, else the files are put back as they were.

use crate::source::{self, content, copy, expand, fields, normalize, read, resolve_item, sget, RANK_KEYS};
use aegis_core::json::{self, gstr, Dict};
use serde_json::Value;
use std::path::{Path, PathBuf};

// ---- JSON.stringify(doc, "\t", false) ------------------------------------------------------------------------

/// Godot's JSON.stringify with a tab indent (keys in insertion order) plus the trailing newline the save adds.
pub fn stringify(v: &Value) -> String {
    let mut out = String::new();
    write(&mut out, v, 0);
    out.push('\n');
    out
}

fn write(out: &mut String, v: &Value, depth: usize) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => match n.as_i64() {
            Some(i) => out.push_str(&i.to_string()),
            None => out.push_str(&float_text(n.as_f64().unwrap_or(0.0))),
        },
        Value::String(s) => quote(out, s),
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                indent(out, depth + 1);
                write(out, x, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push(']');
        }
        Value::Object(o) => {
            if o.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, x)) in o.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                indent(out, depth + 1);
                quote(out, k);
                out.push_str(": ");
                write(out, x, depth + 1);
            }
            out.push('\n');
            indent(out, depth);
            out.push('}');
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push('\t');
    }
}

/// String::json_escape.
fn quote(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// JSON::stringify of a float: 14 significant digits (`String::num`, trailing zeros trimmed to one decimal).
fn float_text(x: f64) -> String {
    if x == 0.0 {
        return "0.0".into();
    }
    let precision = (14 - x.abs().log10().floor() as i64).max(1) as usize;
    let mut t = format!("{x:.precision$}");
    if t.contains('.') {
        while t.ends_with('0') && !t.ends_with(".0") {
            t.pop();
        }
    }
    t
}

// ---- Godot Variant equality ------------------------------------------------------------------------------------

/// GDScript `==` on JSON-shaped values: 1 == 1.0; arrays element-wise; dictionaries by content.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w))),
        _ => a == b,
    }
}

// ---- Storage forms -----------------------------------------------------------------------------------------------

/// ContentSchema.is_visible: a field with a `when` {key, in | not_in} shows only for matching entries.
fn visible(f: &Dict, full: &Dict) -> bool {
    let Some(cond) = f.get("when").and_then(Value::as_object) else { return true };
    let value = sget(full, &json::s(cond, "key"), "");
    if let Some(allowed) = cond.get("in").and_then(Value::as_array) {
        return allowed.iter().any(|v| gstr(v) == value);
    }
    if let Some(denied) = cond.get("not_in").and_then(Value::as_array) {
        return !denied.iter().any(|v| gstr(v) == value);
    }
    true
}

fn has_field(section: &str, kind: &str, key: &str) -> bool {
    fields(section, kind).iter().any(|f| json::s(json::dict(f), "key") == key)
}

/// ContentSchema.to_storage of a normalized row: hidden and internal fields, `system` false and `origin` system
/// left out; unknown keys kept.
fn content_storage(kind: &str, full: &Dict) -> Dict {
    let mut out = Dict::new();
    for f in fields("content", kind) {
        let f = json::dict(f);
        let key = json::s(f, "key");
        if !visible(f, full) || json::b(f, "internal", false) {
            continue;
        }
        let v = json::get(full, &key);
        if (key == "system" && !json::to_bool(v)) || (key == "origin" && gstr(v) == "system") {
            continue;
        }
        out.insert(key, v.clone());
    }
    for (k, v) in full {
        if !out.contains_key(k) && !has_field("content", kind, k) {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

/// WorldSchema.to_storage of a raw entry (normalized first; an empty `override` is left out).
pub fn world_storage(kind: &str, entry: &Dict) -> Dict {
    let full = normalize(fields("world", kind), entry, true);
    let mut out = Dict::new();
    for f in fields("world", kind) {
        let key = json::s(json::dict(f), "key");
        if key == "override" && sget(&full, &key, "").is_empty() {
            continue;
        }
        out.insert(key.clone(), json::get(&full, &key).clone());
    }
    for (k, v) in &full {
        if !out.contains_key(k) && !has_field("world", kind, k) {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

const ITEM_DERIVED: [&str; 13] = ["item_type", "weapon_type", "two_handed", "off_hand_category", "base_power", "base_range", "block_cost", "burden", "mitigation", "required_stats", "stat_bonuses", "effects", "threshold_tiers"];

/// SkillKinds.has_field: common fields, the ATTACK-only stance override, and the fields of the kind's sections.
fn skill_has_field(kind: &str, key: &str) -> bool {
    const COMMON: [&str; 16] = ["id", "name", "kind", "level", "blocks", "description", "template", "system", "basic", "learned_only", "grade", "requires", "origin", "unlock_offset", "family", "tier"];
    if COMMON.contains(&key) {
        return true;
    }
    if key == "stance" {
        return kind == "ATTACK";
    }
    let sections: &[&str] = match kind {
        "ATTACK" => &["damage", "inflict", "combo", "flow", "reach", "area", "gate"],
        "DEFEND" => &["grant", "guard", "flow", "gate"],
        "FOCUS" => &["focus", "flow"],
        "SUPPORT" => &["reach", "support", "grant", "flow", "gate"],
        "DISENGAGE" => &["jump"],
        "REST" => &["rest"],
        _ => &[],
    };
    sections.iter().any(|s| {
        let f: &[&str] = match *s {
            "damage" => &["power_mult", "weapon_hand", "dash_tiles"],
            "inflict" => &["status_effect", "status_potency", "status_magnitude", "status_rounds"],
            "reach" => &["reach_min", "reach_max"],
            "area" => &["area"],
            "support" => &["target_mode", "heal_stat", "heal_base", "heal_mult", "cleanse_statuses"],
            "grant" => &["grant_status", "status_magnitude", "status_rounds", "extra_grants"],
            "guard" => &["mitigation"],
            "focus" => &["boost", "boost_rounds"],
            "jump" => &["jump_tiles"],
            "flow" => &["ends_in_stance"],
            "combo" => &["disrupt", "exploit", "exploit_consume"],
            "rest" => &["recover_ap"],
            "gate" => &["weapon_types", "min_proficiency"],
            _ => &[],
        };
        f.contains(&key)
    })
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
            if !same(json::get(row, k), json::get(first, k)) {
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
        if !CORE.contains(&key.as_str()) && same(&value, json::get(f, "default")) {
            continue;
        }
        out.insert(key, value);
    }
    for (k, v) in first {
        if !out.contains_key(k) && !has_field("content", "skills", k) {
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
        _ => Ok(content_storage(kind, &content(kind, entry))),
    }
}

// ---- Files -------------------------------------------------------------------------------------------------------

/// Where an entry of `kind` lives: data/catalog/<kind>.json, or data/worlds/<world>/<kind>.json.
pub fn file_of(data_dir: &Path, world: Option<&str>, kind: &str) -> PathBuf {
    match (world, game_kind(kind)) {
        (Some(w), _) => data_dir.join("worlds").join(w).join(format!("{kind}.json")),
        (None, Some((_, file, _, _))) => data_dir.join(file),
        (None, None) => data_dir.join("catalog").join(format!("{kind}.json")),
    }
}

/// The game-data files beside the catalogs (MapCatalog, QuestCatalog, ServiceCatalog): kind, file, the file's
/// "kind", the stored fields (to_storage keeps exactly these) and the normalize.
type GameKind = (&'static str, &'static str, &'static str, &'static [&'static str]);
pub const GAME_KINDS: [GameKind; 3] = [
    ("maps", "maps/maps.json", "map_templates", &["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "tags"]),
    ("quests", "quests/quests.json", "quest_templates", &["id", "name", "description", "objectives", "rewards", "tags"]),
    ("services", "dialogue/services.json", "dialogue_services", &["id", "text"]),
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
    let doc = read(&data_dir.join("catalog/item_families.json"), &mut errors).map(|d| source::entries_of(d).0).unwrap_or_default();
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
    let mut doc = Dict::new();
    doc.insert("kind".into(), Value::from(game.map(|g| g.2).unwrap_or(kind)));
    doc.insert("version".into(), json::int(version_of(world, kind)));
    let families = derived_families(data_dir);
    let mut stored = Vec::new();
    for e in entries {
        stored.push(Value::Object(match (world, game) {
            (Some(_), _) => world_storage(kind, e),
            (None, Some(_)) => game_storage(kind, e),
            (None, None) => catalog_storage(kind, e, &families)?,
        }));
    }
    doc.insert("entries".into(), Value::Array(stored));
    Ok(stringify(&Value::Object(doc)))
}

/// The raw entries of a data file (numbers as JSON.parse reads them).
fn raw_entries(data_dir: &Path, world: Option<&str>, kind: &str) -> Vec<Dict> {
    let mut errors = Vec::new();
    read(&file_of(data_dir, world, kind), &mut errors).map(|d| source::entries_of(d).0).unwrap_or_default().iter().filter_map(Value::as_object).cloned().collect()
}

/// The file text a save of `kind` would write now (tests: equal to the file when nothing changed).
pub fn resave_text(data_dir: &Path, world: Option<&str>, kind: &str) -> Result<String, String> {
    rewrite(data_dir, world, kind, &raw_entries(data_dir, world, kind))
}

/// The world manifest (world.json) as WorldCatalog.save_world writes it.
pub fn manifest_text(data_dir: &Path, world: &str) -> Result<String, String> {
    let path = data_dir.join("worlds").join(world).join("world.json");
    let mut errors = Vec::new();
    let raw = read(&path, &mut errors).and_then(|v| v.as_object().cloned()).ok_or_else(|| format!("{}: missing or not an object", path.display()))?;
    let mut m = normalize(fields("world", "world"), &raw, true);
    m.insert("id".into(), Value::from(world));
    let mut doc = world_storage("world", &m);
    doc.insert("version".into(), json::int(1));
    Ok(stringify(&Value::Object(doc)))
}

// ---- Upsert / remove ---------------------------------------------------------------------------------------------

fn is_id(id: &str) -> bool {
    let mut c = id.chars();
    c.next().is_some_and(|f| f.is_ascii_lowercase()) && c.all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '_')
}

/// Schema checks of a normalized entry (ContentSchema.field_errors / WorldSchema shape): id, name, enum options.
fn field_errors(section: &str, kind: &str, full: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let id = sget(full, "id", "");
    if id.is_empty() {
        errors.push("ID is required.".into());
    } else if !is_id(&id) {
        errors.push(format!("ID '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
    }
    if has_field(section, kind, "name") && sget(full, "name", "").trim().is_empty() {
        errors.push("Name is required.".into());
    }
    for f in fields(section, kind) {
        let f = json::dict(f);
        if json::s(f, "type") != "enum" || !visible(f, full) {
            continue;
        }
        if let Some(options) = f.get("options").and_then(Value::as_array) {
            let value = sget(full, &json::s(f, "key"), "");
            if !options.iter().any(|o| gstr(o) == value) {
                errors.push(format!("{}: '{value}' is not one of {}.", json::s(f, "label"), options.iter().map(gstr).collect::<Vec<_>>().join(", ")));
            }
        }
    }
    errors
}

/// What an upsert or remove did.
pub struct Outcome {
    pub file: PathBuf,
    pub replaced: bool,
}

/// ContentCatalog.upsert / WorldCatalog.upsert: validates `entry` (stored shape), replaces the entry with its id (or
/// `replace_id`) or appends it, writes the file, and keeps it only when the whole data folder still loads cleanly.
pub fn upsert(data_dir: &Path, world: Option<&str>, kind: &str, entry: &Dict, replace_id: Option<&str>) -> Result<Outcome, Vec<String>> {
    let known: &[&str] = if world.is_some() { &source::WORLD_KINDS } else { &["classes", "units", "skills", "items", "item_families", "passives", "encounters", "terrain", "maps", "quests", "services"] };
    if !known.contains(&kind) {
        return Err(vec![format!("Unknown kind '{kind}' (one of {}).", known.join(", "))]);
    }
    if let Some(w) = world {
        if !data_dir.join("worlds").join(w).join("world.json").is_file() {
            return Err(vec![format!("Unknown world '{w}'.")]);
        }
    }
    let section = if world.is_some() { "world" } else { "content" };
    let entry = aegis_core::json::normalize_dict(entry);
    let game = world.is_none() && game_kind(kind).is_some();
    let full = if world.is_some() { normalize(fields("world", kind), &entry, true) } else if game { game_storage(kind, &entry) } else { content(kind, &entry) };
    let errors = if game { field_errors("none", kind, &full) } else { field_errors(section, kind, &full) };
    if !errors.is_empty() {
        return Err(errors);
    }
    let id = sget(&full, "id", "");
    let target = replace_id.unwrap_or(&id).to_string();
    let mut entries = raw_entries(data_dir, world, kind);
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
    let outcome = Outcome { file: file_of(data_dir, world, kind), replaced: at.is_some() };
    commit(data_dir, world, kind, &entries)?;
    Ok(outcome)
}

/// Removes the entry `id` of `kind` (no reference check beyond the data folder still loading cleanly).
pub fn remove(data_dir: &Path, world: Option<&str>, kind: &str, id: &str) -> Result<Outcome, Vec<String>> {
    let mut entries = raw_entries(data_dir, world, kind);
    let Some(i) = entries.iter().position(|e| sget(e, "id", "") == id) else { return Err(vec![format!("No {kind} entry '{id}'.")]) };
    entries.remove(i);
    let outcome = Outcome { file: file_of(data_dir, world, kind), replaced: true };
    commit(data_dir, world, kind, &entries)?;
    Ok(outcome)
}

/// Writes the file, then reloads the data folder: new load problems or broken references put the file back.
fn commit(data_dir: &Path, world: Option<&str>, kind: &str, entries: &[Dict]) -> Result<(), Vec<String>> {
    let path = file_of(data_dir, world, kind);
    let before = std::fs::read_to_string(&path).ok();
    let old_problems = check(data_dir);
    let text = rewrite(data_dir, world, kind, entries).map_err(|e| vec![e])?;
    std::fs::write(&path, text).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    let new: Vec<String> = check(data_dir).into_iter().filter(|p| !old_problems.contains(p)).collect();
    if !new.is_empty() {
        match before {
            Some(t) => std::fs::write(&path, t).ok(),
            None => std::fs::remove_file(&path).ok(),
        };
        return Err(new);
    }
    Ok(())
}

/// Load problems of the data folder plus references (schema `ref` / `refs` fields) that name no catalog entry.
pub fn check(data_dir: &Path) -> Vec<String> {
    let src = source::load(data_dir);
    let mut out = src.errors;
    let ids = |kind: &str| -> Vec<String> {
        let rows = if kind == "item_families" { &src.game["item_families"] } else { &src.catalog[kind] };
        rows.as_array().map(|a| a.iter().map(|e| json::s(json::dict(e), "id")).collect()).unwrap_or_default()
    };
    let cat = src.catalog.as_object().cloned().unwrap_or_default();
    for (kind, rows) in &cat {
        for f in fields("content", kind) {
            let f = json::dict(f);
            let target = json::s(f, "ref");
            if target.is_empty() || !matches!(json::s(f, "type").as_str(), "ref" | "refs") || !source::CATALOG_KINDS.contains(&target.as_str()) && target != "item_families" {
                continue;
            }
            let known = ids(&target);
            let key = json::s(f, "key");
            for e in json::arr(rows) {
                let e = json::dict(e);
                if !visible(f, e) {
                    continue;
                }
                let v = json::get(e, &key);
                let refs: Vec<String> = if v.is_array() { json::arr(v).iter().map(gstr).collect() } else { vec![gstr(v)] };
                for r in refs.into_iter().filter(|r| !r.is_empty() && r != "<null>" && r != "none") {
                    if !known.contains(&r) {
                        out.push(format!("{kind} '{}': {} names the unknown {target} '{r}'.", json::s(e, "id"), json::s(f, "label")));
                    }
                }
            }
        }
    }
    out
}
