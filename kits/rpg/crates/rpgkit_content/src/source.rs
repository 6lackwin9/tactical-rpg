//! The game's data loader on the engine content layer (`aegis_content`: file format, schema normalize, world graph):
//! reads the project's `data/` folder the way Godot's catalogs read it and builds the resolved values (catalog, worlds,
//! game data) `Data` is built from. Read paths only (editing and
//! validation stay out): ContentCatalog.reload_from_disk / _read_kind (SkillRanks.expand with SkillText.render,
//! ItemModifiers.resolve, ItemFamilies.derive), WorldCatalog.reload_from_disk (the engine's world loader over
//! WORLD_KINDS), MapCatalog / QuestCatalog / ServiceCatalog (normalize) and XpCurve.load_file. The field tables are
//! kits/rpg/schema.json, exported once from Godot's ContentSchema / WorldSchema (the retired parity exporter).
//! Like Godot, a bad file or entry is reported in `errors` and skipped, never fatal.

use aegis_content::file::{entries_of, read};
use aegis_content::schema::{normalize, sget, snapped};
use aegis_core::json::{self, gstr, int, to_float_or, to_int_or, Dict};
use rpgkit_rules::status;
use serde_json::{Map, Value};
use std::path::Path;
use std::sync::OnceLock;

/// The resolved values (export shapes) and the load problems (Godot's load_errors).
pub struct Source {
    pub catalog: Value,
    pub worlds: Value,
    pub game: Value,
    pub errors: Vec<String>,
}

/// Catalog kinds in the order the catalog export lists them (item families go with the game data).
pub const CATALOG_KINDS: [&str; 7] = ["classes", "units", "skills", "items", "passives", "encounters", "terrain"];
/// DERIVE1: the derived-stat rows, loaded into the catalog beside CATALOG_KINDS (Godot had no such kind).
pub const DERIVED_KIND: &str = crate::check::derived::KIND;
/// RULESET1-R: the ruleset rows, loaded into the catalog beside CATALOG_KINDS after the skills and derived stats they
/// name (Godot had no such kind).
pub const RULESET_KIND: &str = crate::check::ruleset::KIND;
/// PORTRAITFX1-S: the portrait cue rows, loaded into the catalog beside CATALOG_KINDS after the derived stats they name
/// (Godot had no such kind).
pub const PORTRAIT_KIND: &str = crate::check::portrait::KIND;
/// PORTRAITFX1-EX: the expression-effect rows, loaded into the catalog beside CATALOG_KINDS after the portrait cues
/// (Godot had no such kind).
pub const EXPRESSION_FX_KIND: &str = crate::check::expression_fx::KIND;
/// HOOKS1-S: the combat hook kinds (ground hazards, unit meters, data zones), loaded into the catalog beside
/// CATALOG_KINDS after the skills the meters name, in this order (Godot had no such kinds).
pub const HOOK_KINDS: [&str; 3] = [rpgkit_data::HAZARDS_KIND, rpgkit_data::UNIT_METERS_KIND, rpgkit_data::ZONES_KIND];
/// TERMS1-L: the game's term rows, loaded into the catalog beside CATALOG_KINDS after the hook kinds (Godot had no
/// such kind).
pub const TERMS_KIND: &str = crate::check::terms::KIND;
/// REGIONGROUND1 KT 1.1: the ground rows, loaded into the catalog beside CATALOG_KINDS after the terms (Godot had no
/// such kind).
pub const GROUNDS_KIND: &str = crate::check::ground::KIND;
/// WorldSchema.KINDS, then CAMPAIGN1's `meters`, `checkpoints` and `triggers` (11.2) and STORYUI1's `vignettes`,
/// `sequences` and `boards` (4.1-4.4; Godot had no such kinds).
pub const WORLD_KINDS: [&str; 19] = [
    "regions", "locations", "factions", "characters", "items", "disciplines", "quests", "lore", "encounters", "loot", "dialogues", "shops", "routes", "meters", "checkpoints", "triggers", "vignettes", "sequences", "boards",
];

const SCHEMA_TEXT: &str = include_str!("../../../schema.json");

/// kits/rpg/schema.json: {"content": {kind: [field]}, "world": {kind: [field]}}.
pub fn schema() -> &'static Value {
    static S: OnceLock<Value> = OnceLock::new();
    S.get_or_init(|| serde_json::from_str(SCHEMA_TEXT).expect("kits/rpg/schema.json"))
}

pub(crate) fn fields(section: &str, kind: &str) -> &'static [Value] {
    aegis_content::schema::fields(schema(), section, kind)
}

pub fn load(data_dir: &Path) -> Source {
    let mut errors = Vec::new();
    let catalog = load_catalog(data_dir, &mut errors);
    let worlds = aegis_content::world::load(data_dir, schema(), &WORLD_KINDS, &mut errors);
    let mut game = Dict::new();
    game.insert("maps".into(), Value::Array(load_list(&data_dir.join("maps/maps.json"), &mut errors, map_template)));
    game.insert("quests".into(), Value::Array(load_list(&data_dir.join("quests/quests.json"), &mut errors, quest_template)));
    game.insert("services".into(), Value::Array(load_list(&data_dir.join("dialogue/services.json"), &mut errors, service)));
    let curve = xp_curve(&data_dir.join("progression/xp_curve.json"), &mut errors);
    let steps = json::aget(&curve, "xp_to_next");
    let xp: Vec<Value> = (1..=rpgkit_rules::stats::MAX_LEVEL as usize).map(|level| int(steps.iter().take(level - 1).map(json::to_int).sum())).collect();
    game.insert("xp_for_level".into(), Value::Array(xp));
    game.insert("xp_curve".into(), Value::Object(curve));
    game.insert("item_families".into(), catalog["item_families"].clone());
    let mut cat = catalog.as_object().cloned().unwrap_or_default();
    cat.shift_remove("item_families");
    Source { catalog: Value::Object(cat), worlds, game: Value::Object(game), errors }
}

/// ContentSchema.normalize of a catalog entry of `kind`.
pub(crate) fn content(kind: &str, entry: &Dict) -> Dict {
    normalize(fields("content", kind), entry, false)
}

// ---- ContentCatalog --------------------------------------------------------------------------------------------

fn load_catalog(data_dir: &Path, errors: &mut Vec<String>) -> Value {
    let dir = data_dir.join("catalog");
    let mut out = Dict::new();
    let families = read_kind(&dir, "item_families", &[], errors);
    for kind in CATALOG_KINDS {
        let mut rows = read_kind(&dir, kind, &families, errors);
        if kind == "skills" {
            mark_spell_sources(&mut rows, &families);
        }
        out.insert(kind.into(), Value::Array(rows.into_iter().map(Value::Object).collect()));
    }
    out.insert(DERIVED_KIND.into(), Value::Array(read_derived(&dir, errors).into_iter().map(Value::Object).collect()));
    let rulesets = read_rulesets(&dir, &out, errors);
    out.insert(RULESET_KIND.into(), Value::Array(rulesets.into_iter().map(Value::Object).collect()));
    // PORTRAITFX2 FL1: the face rows first, so the cue rows check against the game's faces
    out.insert(EXPRESSION_FX_KIND.into(), Value::Array(read_expression_fx(&dir, errors).into_iter().map(Value::Object).collect()));
    let cues = read_portrait_cues(&dir, &out, errors);
    out.insert(PORTRAIT_KIND.into(), Value::Array(cues.into_iter().map(Value::Object).collect()));
    for kind in HOOK_KINDS {
        let rows = read_hook_kind(&dir, kind, &out, &families, errors);
        out.insert(kind.into(), Value::Array(rows.into_iter().map(Value::Object).collect()));
    }
    out.insert(TERMS_KIND.into(), Value::Array(read_terms(&dir, errors).into_iter().map(Value::Object).collect()));
    out.insert(GROUNDS_KIND.into(), Value::Array(read_grounds(&dir, errors).into_iter().map(Value::Object).collect()));
    out.insert("item_families".into(), Value::Array(families.into_iter().map(Value::Object).collect()));
    Value::Object(out)
}

/// DERIVE1 4.1 (engine-only, beside CATALOG_KINDS like the item families): the derived-stat rows that pass their
/// checks (`check::derived`), in topological order; every problem goes to `errors` and its row is skipped.
fn read_derived(dir: &Path, errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{DERIVED_KIND}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let at = path.display().to_string();
    let mut rows = Vec::new();
    for (i, item) in entries_of(doc).0.iter().enumerate() {
        match item.as_object() {
            Some(d) => rows.push(content(DERIVED_KIND, d)),
            None => errors.push(format!("{at}: entry {} is not an object; it is skipped.", i + 1)),
        }
    }
    let checked = crate::check::derived::check_rows(&rows);
    for (row, found) in rows.iter().zip(&checked.errors) {
        errors.extend(found.iter().map(|m| format!("{at}: derived stat '{}': {m}", sget(row, "id", ""))));
    }
    checked.order.iter().map(|&i| rows[i].clone()).collect()
}

/// RULESET1-R (engine-only, beside CATALOG_KINDS): the ruleset rows that pass their checks (`check::ruleset`) against
/// the loaded skills and derived stats (`loaded`); every problem goes to `errors` and its row is skipped.
fn read_rulesets(dir: &Path, loaded: &Dict, errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{RULESET_KIND}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let at = path.display().to_string();
    let mut rows = Vec::new();
    for (i, item) in entries_of(doc).0.iter().enumerate() {
        match item.as_object() {
            Some(d) => rows.push(content(RULESET_KIND, d)),
            None => errors.push(format!("{at}: entry {} is not an object; it is skipped.", i + 1)),
        }
    }
    let skills = json::arr(json::get(loaded, "skills"));
    let skill_intent = |id: &str| crate::check::ruleset::skill_intent_in(skills, id);
    let derived: Vec<String> = json::arr(json::get(loaded, DERIVED_KIND)).iter().map(|e| json::s(json::dict(e), "id")).collect();
    let checked = crate::check::ruleset::check_rows(&rows, &crate::check::ruleset::Refs { skill_intent: &skill_intent, derived: &derived });
    let mut kept = Vec::new();
    for (row, found) in rows.into_iter().zip(checked) {
        if found.is_empty() {
            kept.push(row);
        } else {
            errors.extend(found.iter().map(|m| format!("{at}: ruleset '{}': {m}", sget(&row, "id", ""))));
        }
    }
    kept
}

/// PORTRAITFX1-S 5.1 (engine-only, beside CATALOG_KINDS): the portrait cue rows that pass their checks
/// (`check::portrait`) against the loaded derived stats and faces (`loaded`; PORTRAITFX2 FL1), in file order; every
/// problem goes to `errors` and its row is skipped.
fn read_portrait_cues(dir: &Path, loaded: &Dict, errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{PORTRAIT_KIND}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let at = path.display().to_string();
    let mut rows = Vec::new();
    for (i, item) in entries_of(doc).0.iter().enumerate() {
        match item.as_object() {
            Some(d) => rows.push(content(PORTRAIT_KIND, d)),
            None => errors.push(format!("{at}: entry {} is not an object; it is skipped.", i + 1)),
        }
    }
    let derived: Vec<String> = json::arr(json::get(loaded, DERIVED_KIND)).iter().map(|e| json::s(json::dict(e), "id")).collect();
    let faces = rpgkit_data::faces_of(json::arr(json::get(loaded, EXPRESSION_FX_KIND)).iter().map(|e| json::get(json::dict(e), "id").as_str().unwrap_or("")));
    let checked = crate::check::portrait::check_rows(&rows, &crate::check::portrait::Refs { derived: &derived, faces: &faces });
    let mut kept = Vec::new();
    for (row, found) in rows.into_iter().zip(checked) {
        if found.is_empty() {
            kept.push(row);
        } else {
            errors.extend(found.iter().map(|m| format!("{at}: portrait cue '{}': {m}", sget(&row, "id", ""))));
        }
    }
    kept
}

/// PORTRAITFX1-EX (engine-only, beside CATALOG_KINDS): the expression-effect rows that pass their checks
/// (`check::expression_fx`), in file order; every problem goes to `errors` and its row is skipped. A missing file is an
/// empty table.
fn read_expression_fx(dir: &Path, errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{EXPRESSION_FX_KIND}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let at = path.display().to_string();
    let mut rows = Vec::new();
    for (i, item) in entries_of(doc).0.iter().enumerate() {
        match item.as_object() {
            Some(d) => rows.push(content(EXPRESSION_FX_KIND, d)),
            None => errors.push(format!("{at}: entry {} is not an object; it is skipped.", i + 1)),
        }
    }
    let checked = crate::check::expression_fx::check_rows(&rows);
    let mut kept = Vec::new();
    for (row, found) in rows.into_iter().zip(checked) {
        if found.is_empty() {
            kept.push(row);
        } else {
            errors.extend(found.iter().map(|m| format!("{at}: expression effect '{}': {m}", sget(&row, "id", ""))));
        }
    }
    kept
}

/// TERMS1-L 5.1 (engine-only, beside CATALOG_KINDS): the term rows that pass their checks (`check::terms`), in file
/// order; every problem goes to `errors` and its row is skipped. A missing file is an empty table.
fn read_terms(dir: &Path, errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{TERMS_KIND}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let at = path.display().to_string();
    let mut rows = Vec::new();
    for (i, item) in entries_of(doc).0.iter().enumerate() {
        match item.as_object() {
            Some(d) => rows.push(content(TERMS_KIND, d)),
            None => errors.push(format!("{at}: entry {} is not an object; it is skipped.", i + 1)),
        }
    }
    let checked = crate::check::terms::check_rows(&rows);
    let mut kept = Vec::new();
    for (row, found) in rows.into_iter().zip(checked) {
        if found.is_empty() {
            kept.push(row);
        } else {
            errors.extend(found.iter().map(|m| format!("{at}: term '{}': {m}", sget(&row, "id", ""))));
        }
    }
    kept
}

/// REGIONGROUND1 KT 1.1 (engine-only, beside CATALOG_KINDS): the ground rows that pass their checks (`check::ground`), in
/// file order; every problem goes to `errors` and its row is skipped. A missing file is an empty table.
fn read_grounds(dir: &Path, errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{GROUNDS_KIND}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let at = path.display().to_string();
    let mut rows = Vec::new();
    for (i, item) in entries_of(doc).0.iter().enumerate() {
        match item.as_object() {
            Some(d) => rows.push(content(GROUNDS_KIND, d)),
            None => errors.push(format!("{at}: entry {} is not an object; it is skipped.", i + 1)),
        }
    }
    let checked = crate::check::ground::check_rows(&rows);
    let mut kept = Vec::new();
    for (row, found) in rows.into_iter().zip(checked) {
        if found.is_empty() {
            kept.push(row);
        } else {
            errors.extend(found.iter().map(|m| format!("{at}: ground '{}': {m}", sget(&row, "id", ""))));
        }
    }
    kept
}

/// HOOKS1-S (engine-only, beside CATALOG_KINDS): the rows of a combat hook kind that pass their checks
/// (`check::hook_kinds`) against the loaded skills (`loaded`) and the item families (U3.2-S2: the gear tokens), in file
/// order; every problem goes to `errors` and its row is skipped.
fn read_hook_kind(dir: &Path, kind: &str, loaded: &Dict, families: &[Dict], errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{kind}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let at = path.display().to_string();
    let mut rows = Vec::new();
    for (i, item) in entries_of(doc).0.iter().enumerate() {
        match item.as_object() {
            Some(d) => rows.push(content(kind, d)),
            None => errors.push(format!("{at}: entry {} is not an object; it is skipped.", i + 1)),
        }
    }
    let skills: Vec<Dict> = json::arr(json::get(loaded, "skills")).iter().map(|e| json::dict(e).clone()).collect();
    let derived: Vec<String> = json::arr(json::get(loaded, DERIVED_KIND)).iter().map(|e| json::s(json::dict(e), "id")).collect();
    let checked = crate::check::hook_kinds::check_rows(kind, &rows, &crate::check::hook_kinds::Refs { skills: &skills, derived: &derived, families });
    let mut kept = Vec::new();
    for (row, found) in rows.into_iter().zip(checked) {
        if found.is_empty() {
            kept.push(row);
        } else {
            errors.extend(found.iter().map(|m| format!("{at}: {} '{}': {m}", crate::check::hook_kinds::noun(kind), sget(&row, "id", ""))));
        }
    }
    kept
}

/// U3.2-S2: the gear tokens whose family has engine rule `rule`, in table order.
pub fn families_with_rule(families: &[Dict], rule: &str) -> Vec<String> {
    families.iter().filter(|f| json::strings(json::get(f, "engine_rules")).iter().any(|r| r == rule)).map(|f| sget(f, "id", "")).collect()
}

/// U3.2-S2: each skill row says whether one of its `weapon_types` is a family with the engine rule `spell_source`
/// (`rpgkit_data::SKILL_SPELL_SOURCE`, read by the passive group "spell"); no family table leaves the rows as they are.
pub(crate) fn mark_spell_sources(rows: &mut [Dict], families: &[Dict]) {
    if families.is_empty() {
        return;
    }
    let sources = families_with_rule(families, rpgkit_data::SPELL_SOURCE_RULE);
    for row in rows {
        let spell = json::strings(json::get(row, "weapon_types")).iter().any(|w| sources.contains(w));
        row.insert(rpgkit_data::SKILL_SPELL_SOURCE.into(), Value::Bool(spell));
    }
}

/// ContentCatalog._read_kind.
fn read_kind(dir: &Path, kind: &str, families: &[Dict], errors: &mut Vec<String>) -> Vec<Dict> {
    let path = dir.join(format!("{kind}.json"));
    let Some(doc) = read(&path, errors) else { return Vec::new() };
    let (list, version) = entries_of(doc);
    let at = path.display().to_string();
    if kind == "skills" {
        if version < 2 {
            errors.push(format!("{at}: version {version} skills files (flat rows) are no longer read; write version 2 (ranked entries)."));
            return Vec::new();
        }
        return ranked_skills(&list, &at, errors);
    }
    if kind == "items" {
        return read_items(&list, &at, version, families, errors);
    }
    let mut found = Vec::new();
    for item in &list {
        let Some(d) = item.as_object() else { continue };
        if kind == "classes" && d.get("skill_path").is_some_and(Value::is_array) {
            errors.push(format!("{at}: class '{}' stores a pre-CLASS1 skill_path list; convert it in Godot first.", sget(d, "id", "")));
        }
        let row = content(kind, d);
        found.push(if kind == "item_families" { derive_family(&row) } else { row });
    }
    found
}

// ---- Skills (SkillRanks, SkillText) ----------------------------------------------------------------------------

/// The per-rank skill fields (SkillRanks.RANK_KEYS; RULESET1-S adds the timeline's `ap_cost`, `recovery_blocks`,
/// `stagger` and `windup_poise`, HOOKS1-S the costs `meters` and `hp_cost`).
pub(crate) const RANK_KEYS: [&str; 26] = [
    "level", "blocks", "unlock_offset", "power_mult", "dash_tiles", "reach_min", "reach_max", "status_potency", "status_magnitude", "status_rounds", "heal_base", "heal_mult", "grant_status", "extra_grants",
    "cleanse_statuses", "mitigation", "boost", "boost_rounds", "jump_tiles", "recover_ap", "ap_cost", "recovery_blocks", "stagger", "windup_poise", "meters", "hp_cost",
];

/// ContentCatalog._read_ranked_skills: every entry expanded into its rank rows; all rank 1 rows first, then rank 2...
fn ranked_skills(list: &[Value], at: &str, errors: &mut Vec<String>) -> Vec<Dict> {
    let mut by_rank: Vec<Vec<Dict>> = vec![Vec::new(); 5];
    for item in list {
        let Some(d) = item.as_object() else {
            errors.push(format!("{at}: an entry is not an object; it is skipped."));
            continue;
        };
        let id = sget(d, "id", "");
        let mut bad = false;
        for banned in ["family", "tier", "template"] {
            if d.contains_key(banned) {
                errors.push(format!("{at}: skill '{id}' stores '{banned}'; version 2 entries derive it."));
                bad = true;
            }
        }
        for m in rank_mods(d) {
            for banned in ["name", "family", "tier", "id"] {
                if m.contains_key(banned) {
                    errors.push(format!("{at}: skill '{id}' has a rank modifier with '{banned}'; rank names and ids are derived."));
                    bad = true;
                }
            }
        }
        if bad {
            continue;
        }
        let mods = rank_mods(d);
        for (i, mut row) in expand(d).into_iter().enumerate() {
            unset_default_mitigation(&mut row, d, &mods, i);
            by_rank[i.min(4)].push(row);
        }
    }
    by_rank.into_iter().flatten().collect()
}

/// AUDIT-11 (S1.2-S4, FB-85): a loaded DEFEND rank row whose entry authors no `mitigation` (neither the entry nor this
/// rank's modifier) loses the schema default the normalisation filled, so the battle takes the ruleset's guard fraction
/// for it (`rpgkit_battle::skill::guard_mitigation`). Only the loader drops it: the writer (`store`) expands the same
/// entries with the default kept, so a resave is unchanged. `index` is the rank row's 0-based position.
fn unset_default_mitigation(row: &mut Dict, entry: &Dict, mods: &[Dict], index: usize) {
    let authored = entry.contains_key("mitigation") || index.checked_sub(1).and_then(|m| mods.get(m)).is_some_and(|m| m.contains_key("mitigation"));
    if !authored && sget(row, "kind", "") == "DEFEND" {
        row.shift_remove("mitigation");
    }
}

fn rank_mods(e: &Dict) -> Vec<Dict> {
    json::aget(e, "ranks").iter().map(|r| r.as_object().cloned().unwrap_or_default()).collect()
}

fn roman(n: usize) -> &'static str {
    ["", "I", "II", "III", "IV", "V"][n.min(5)]
}

/// SkillRanks.expand.
pub(crate) fn expand(e: &Dict) -> Vec<Dict> {
    let mods = rank_mods(e);
    let mut base_raw = e.clone();
    for k in ["ranks", "family", "tier", "template"] {
        base_raw.shift_remove(k);
    }
    let base = content("skills", &base_raw);
    let head = sget(&base, "id", "");
    let mut rows = Vec::new();
    for rank in 1..=mods.len() + 1 {
        // SkillRanks.effective
        let mut row = base.clone();
        if rank >= 2 {
            for (k, v) in &mods[rank - 2] {
                if RANK_KEYS.contains(&k.as_str()) {
                    row.insert(k.clone(), v.clone());
                }
            }
        }
        let mut row = content("skills", &row);
        row.insert("id".into(), Value::from(if rank <= 1 { head.clone() } else { format!("{head}_{rank}") }));
        row.insert("family".into(), Value::from(if rank <= 1 { String::new() } else { head.clone() }));
        row.insert("tier".into(), int(rank as i64));
        let base_name = sget(&base, "name", "");
        row.insert("name".into(), Value::from(if rank <= 1 { base_name } else { format!("{base_name} {}", roman(rank)) }));
        let template = sget(&base, "description", "");
        row.insert("template".into(), Value::from(template.clone()));
        // SkillRanks.resolve
        let literal = if rank >= 2 { mods[rank - 2].get("description").filter(|v| !v.is_null()).map(gstr) } else { None };
        let kind = sget(&row, "kind", "");
        // A1.2-S2: the catalog text is ruleset-free, so it renders the stance tokens (stances on)
        let text = literal.unwrap_or_else(|| render(&template, &kind, &row, rank, true));
        row.insert("description".into(), Value::from(text));
        rows.push(row);
    }
    rows
}

/// SkillText.TOKEN_KINDS.
fn token_kinds(token: &str) -> Option<&'static [&'static str]> {
    Some(match token {
        "name" | "ap" | "blocks" | "unlock" => &["ALL"],
        "power" | "dash" | "potency" | "exploit" | "disrupt" => &["ATTACK"],
        "reach" => &["ATTACK", "SUPPORT"],
        "weapons" | "proficiency" => &["ATTACK", "DEFEND", "SUPPORT"],
        "status" | "magnitude" => &["ATTACK", "SUPPORT", "DEFEND"],
        "rounds" | "flow" => &["ATTACK", "SUPPORT", "DEFEND", "FOCUS"],
        "heal" | "cleanse" => &["SUPPORT"],
        "mitigation" => &["DEFEND"],
        "boost" => &["FOCUS"],
        "jump" => &["DISENGAGE"],
        "recover" => &["REST"],
        "extra" => &["DEFEND", "SUPPORT"],
        // RULESET1-S (timeline): poise taken, enemy recovery blocks
        "stagger" => &["ATTACK"],
        "recovery" => &["ATTACK", "SUPPORT", "FOCUS"],
        // INPUT1 MR15 (timeline): a skill's range and the position it dashes to
        "range" => &["ATTACK", "SUPPORT"],
        "position" => &["ATTACK"],
        _ => return None,
    })
}

/// SkillText.render: literal parts as they are, valid tokens by the rank's effective row; `stances` (A1.2-S2: the
/// ruleset's `stances`) off renders the stance tokens (`STANCE_TOKENS`) as nothing.
pub fn render(template: &str, kind: &str, row: &Dict, rank: usize, stances: bool) -> String {
    render_tokens(template, |token, suffix| token_kinds(token).is_some_and(|k| k.contains(&"ALL") || k.contains(&kind)).then(|| token_value(token, suffix, row, rank, stances)))
}

/// A1.2-S2 (A1 "Retire orphaned stance fields"): the skill tokens only the classic stance machinery reads; they render
/// as nothing when a ruleset's `stances` is off.
pub const STANCE_TOKENS: [&str; 2] = ["flow", "disrupt"];

/// SkillText.parse as used by render: `{{` and `}}` are literal braces, `{token}` or `{token:suffix}` is filled by
/// `value` (None, an empty or an unclosed token renders as nothing or as written).
fn render_tokens(template: &str, mut value: impl FnMut(&str, &str) -> Option<String>) -> String {
    let chars: Vec<char> = template.chars().collect();
    let n = chars.len();
    let find = |c: char, from: usize| (from..n).find(|&j| chars[j] == c);
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        let ch = chars[i];
        if ch == '{' {
            if i + 1 < n && chars[i + 1] == '{' {
                out.push('{');
                i += 2;
                continue;
            }
            let close = find('}', i + 1);
            let nested = find('{', i + 1);
            let Some(close) = close.filter(|&c| nested.is_none_or(|nst| nst > c)) else {
                out.push(ch);
                i += 1;
                continue;
            };
            let body: String = chars[i + 1..close].iter().collect();
            if !body.trim().is_empty() {
                let (token, suffix) = match body.split_once(':') {
                    Some((t, s)) => (t.to_string(), s.split(':').next().unwrap_or("").to_string()),
                    None => (body.clone(), String::new()),
                };
                if let Some(text) = value(&token, &suffix) {
                    out.push_str(&text);
                }
            }
            i = close + 1;
        } else if ch == '}' {
            if i + 1 < n && chars[i + 1] == '}' {
                out.push('}');
                i += 2;
            } else {
                i += 1;
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

fn plural(n: i64, unit: &str) -> String {
    format!("{n} {unit}{}", if n == 1 { "" } else { "s" })
}

fn format_percent(x: f64) -> String {
    format!("{}%", json::roundi(x * 100.0))
}

/// SkillText.format_decimal: at least 1 and at most 2 decimals.
fn format_decimal(x: f64) -> String {
    let mut text = format!("{:.2}", snapped(x, 0.01));
    while text.ends_with('0') && text.len() - text.find('.').unwrap_or(0) > 2 {
        text.pop();
    }
    text
}

fn format_mult(x: f64) -> String {
    format!("x{}", format_decimal(x))
}

fn format_reach(lo: i64, hi: i64, bare: bool) -> String {
    if hi <= 0 {
        return String::new();
    }
    if lo > 0 && lo != hi {
        return if bare { format!("{lo}-{hi}") } else { format!("{lo}-{hi} tiles") };
    }
    if bare {
        hi.to_string()
    } else {
        plural(hi, "tile")
    }
}

fn format_magnitude(st: &str, magnitude: i64) -> String {
    status::format_magnitude(st, magnitude as i32)
}

/// One `exploit` entry as the {exploit} token shows it: "x1.3 vs Expose"; HOOKS1 EX1's execute entry "x2.0 at 35% HP or
/// less".
fn exploit_text(r: &Dict) -> String {
    let mult = format_mult(json::f(r, "power_mult", 1.0));
    match r.get("hp_below") {
        Some(pct) => format!("{mult} at {}% HP or less", json::to_int(pct)),
        None => format!("{mult} vs {}", status::display_name(&json::s(r, "status"))),
    }
}

/// INPUT1 MR1, MR15: the range N of a skill row with a position (`reach_max`; 0 = the unit's band, unknown here) or, in
/// the transition, of a Maneuver attack with `dash_tiles` and no position; None: it hits from where it stands.
fn maneuver_dash(row: &Dict) -> Option<i64> {
    let maneuver = sget(row, "kind", "") == "ATTACK" && sget(row, "round_intent", "") == "maneuver";
    if !sget(row, "position", "").is_empty() {
        return Some(to_int_or(json::get(row, "reach_max"), 0));
    }
    let dash = to_int_or(json::get(row, "dash_tiles"), 0);
    (maneuver && dash > 0).then_some(dash)
}

/// SkillText.token_value; `stances` off: a stance token (`STANCE_TOKENS`) is "" (A1.2-S2).
pub(crate) fn token_value(token: &str, suffix: &str, row: &Dict, rank: usize, stances: bool) -> String {
    if !stances && STANCE_TOKENS.contains(&token) {
        return String::new();
    }
    let bare = suffix == "n";
    let i = |k: &str, fb: i64| to_int_or(json::get(row, k), fb);
    let f = |k: &str, fb: f64| to_float_or(json::get(row, k), fb);
    let s = |k: &str| sget(row, k, "");
    let counted = |n: i64, unit: &str| if bare { n.to_string() } else { plural(n, unit) };
    match token {
        "name" => s("name"),
        // RULESET1 T10: a set `ap_cost` is the skill's AP (classic rows leave it -1: the level)
        "ap" => {
            let ap = if i("ap_cost", -1) >= 0 { i("ap_cost", -1) } else { i("level", 1) };
            if bare {
                ap.to_string()
            } else {
                format!("{ap} AP")
            }
        }
        "stagger" | "recovery" => {
            let n = i(if token == "stagger" { "stagger" } else { "recovery_blocks" }, 0);
            match (n <= 0, token) {
                (true, _) => String::new(),
                (false, "stagger") => n.to_string(),
                _ => counted(n, "block"),
            }
        }
        "blocks" => counted(i("blocks", 1), "block"),
        "power" => format_percent(f("power_mult", 1.0)),
        "dash" | "jump" => {
            let n = i(if token == "dash" { "dash_tiles" } else { "jump_tiles" }, 0);
            if n <= 0 {
                String::new()
            } else {
                counted(n, "tile")
            }
        }
        "reach" => format_reach(i("reach_min", 0), i("reach_max", 0), bare),
        // INPUT1 MR15: a skill with a position (or the transition's Maneuver dash) renders its range N, else its band
        "range" => match maneuver_dash(row) {
            Some(n) if n > 0 => n.to_string(),
            Some(_) => String::new(),
            None => format_reach(i("reach_min", 0), i("reach_max", 0), true),
        },
        "position" => match (s("position").as_str(), maneuver_dash(row)) {
            ("behind", _) => "behind the target".into(),
            (_, Some(_)) => "to a flank".into(),
            _ => String::new(),
        },
        "weapons" => json::strings(json::get(row, "weapon_types")).join("/"),
        "proficiency" => {
            let level = i("min_proficiency", 0);
            if level > 0 {
                level.to_string()
            } else {
                String::new()
            }
        }
        "status" => {
            let st = if s("kind") == "ATTACK" { s("status_effect") } else { s("grant_status") };
            if st.is_empty() {
                String::new()
            } else {
                status::display_name(&st)
            }
        }
        "potency" => {
            if s("status_effect").is_empty() {
                String::new()
            } else {
                format_percent(f("status_potency", 0.0) / 100.0)
            }
        }
        "magnitude" => {
            let st = if s("status_effect").is_empty() { s("grant_status") } else { s("status_effect") };
            format_magnitude(&st, i("status_magnitude", 0))
        }
        "rounds" => counted(if s("kind") == "FOCUS" { i("boost_rounds", 1) } else { i("status_rounds", 1) }, "round"),
        "heal" => {
            let (base, mult, stat) = (i("heal_base", 0), f("heal_mult", 0.0), s("heal_stat"));
            match (base > 0, mult > 0.0) {
                (true, true) => format!("{base} + {} x {stat}", format_decimal(mult)),
                (true, false) => base.to_string(),
                (false, true) => format!("{} x {stat}", format_decimal(mult)),
                _ => String::new(),
            }
        }
        "cleanse" => {
            let list = json::strings(json::get(row, "cleanse_statuses"));
            if list.iter().any(|x| x == "all") {
                "all statuses".into()
            } else {
                list.join(", ")
            }
        }
        "mitigation" => format_percent(f("mitigation", 0.0)),
        "boost" => format_mult(f("boost", 1.0)),
        "recover" => format!("{} AP", i("recover_ap", 0)),
        "unlock" => {
            let offset = i("unlock_offset", -1);
            format!("+{}", if offset >= 0 { offset } else { rpgkit_rules::stats::TIER_UNLOCK_STEP as i64 * (rank as i64 - 1).max(0) })
        }
        "flow" => match s("ends_in_stance").as_str() {
            "NEUTRAL" => "ends in Neutral".into(),
            "OFFENSIVE" => "ends in Offensive".into(),
            "DEFENSIVE" => "ends in Defensive".into(),
            "TACTICAL" => "ends in Tactical".into(),
            _ => String::new(),
        },
        "exploit" => json::get(row, "exploit").as_array().map(|a| a.iter().map(|r| exploit_text(json::dict(r))).collect::<Vec<_>>().join(", ")).unwrap_or_default(),
        "disrupt" => {
            if s("disrupt") == "STANCE_BREAK" {
                "breaks stance".into()
            } else {
                String::new()
            }
        }
        "extra" => {
            let mut parts = Vec::new();
            for r in json::get(row, "extra_grants").as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
                let r = json::dict(r);
                let st = json::s(r, "status");
                if st.is_empty() {
                    continue;
                }
                let magnitude = format_magnitude(&st, json::i(r, "magnitude", 0));
                let rounds = plural(json::i(r, "rounds", 1), "round");
                parts.push(if magnitude.is_empty() { format!("{} for {rounds}", status::display_name(&st)) } else { format!("{} {magnitude} for {rounds}", status::display_name(&st)) });
            }
            parts.join(", ")
        }
        _ => String::new(),
    }
}

// ---- Items (ItemFamilies, ItemModifiers) -----------------------------------------------------------------------

const HAND_TYPES: [&str; 2] = ["WEAPON", "SHIELD"];
const SLOT_TYPES: [&str; 2] = ["ARMOR", "ACCESSORY"];
const GEAR_TYPES: [&str; 3] = ["WEAPON", "SHIELD", "ARMOR"];
const NUMBER_FIELDS: [(&str, &str, i64); 4] = [("power", "base_power", 0), ("range", "base_range", 1), ("burden", "burden", 0), ("mitigation", "mitigation", 0)];
/// U3.2-S1: number keys gear no longer has (gear `block_cost`: priced in the budget, never read by combat; the wind-up
/// belongs to the skill). The loader ignores them in a family's base and band rows, its adjustable list, an item's adjust
/// modifiers and a stored flat key; the writer stores the cleaned rows, so a resave drops them.
pub const RETIRED_NUMBERS: [&str; 1] = ["block_cost"];

fn is_retired_adjust(m: &Value) -> bool {
    m.as_object().is_some_and(|m| sget(m, "kind", "") == "adjust" && RETIRED_NUMBERS.contains(&sget(m, "field", "").as_str()))
}

pub(crate) fn item_type_of(family: &Dict) -> String {
    sget(family, "item_type", "WEAPON")
}

pub(crate) fn hands_of(family: &Dict) -> Vec<String> {
    if !HAND_TYPES.contains(&item_type_of(family).as_str()) {
        return Vec::new();
    }
    json::strings(json::get(family, "hands"))
}

pub(crate) fn category(family: &Dict) -> &'static str {
    let hands = hands_of(family);
    if hands.is_empty() {
        return "";
    }
    if hands.len() == 1 && hands[0] == "off" {
        return "off";
    }
    if hands.iter().all(|h| h == "main_1h" || h == "main_2h") {
        "main"
    } else {
        ""
    }
}

/// ItemFamilies.off_hand_category: an off-hand family's id is its off-hand category (U3.2-S2: every off-hand family of
/// the table, no fixed list; the content check still names the allowed ones).
pub(crate) fn off_hand_category(family: &Dict) -> String {
    if category(family) == "off" {
        sget(family, "id", "")
    } else {
        String::new()
    }
}

pub(crate) fn default_hand(family: &Dict) -> String {
    let hands = hands_of(family);
    let wanted = sget(family, "default_hand", "");
    if hands.contains(&wanted) {
        return wanted;
    }
    hands.first().cloned().unwrap_or_default()
}

/// ItemFamilies.derive.
pub(crate) fn derive_family(family: &Dict) -> Dict {
    let mut out = family.clone();
    let t = item_type_of(&out);
    if !HAND_TYPES.contains(&t.as_str()) {
        out.insert("hands".into(), Value::Array(Vec::new()));
    }
    if t != "WEAPON" {
        out.insert("scaling_stats".into(), Value::Array(Vec::new()));
    }
    if !SLOT_TYPES.contains(&t.as_str()) {
        out.insert("slots".into(), Value::Array(Vec::new()));
    }
    // U3.2-S1: retired numbers leave the base and band rows and the adjustable list
    for key in ["base", "bands"] {
        if let Some(Value::Object(rows)) = out.get_mut(key) {
            for row in rows.values_mut().filter_map(Value::as_object_mut) {
                for k in RETIRED_NUMBERS {
                    row.shift_remove(k);
                }
            }
        }
    }
    if let Some(Value::Array(list)) = out.get_mut("adjustable") {
        list.retain(|a| !a.as_str().is_some_and(|a| RETIRED_NUMBERS.contains(&a)));
    }
    let cat = off_hand_category(&out);
    out.insert("off_hand_category".into(), Value::from(cat));
    let hand = default_hand(&out);
    out.insert("default_hand".into(), Value::from(hand));
    out
}

/// ContentCatalog._read_items (version 2: every entry resolves against its family).
fn read_items(list: &[Value], at: &str, version: i64, families: &[Dict], errors: &mut Vec<String>) -> Vec<Dict> {
    let mut found = Vec::new();
    if version < 2 {
        errors.push(format!("{at}: version {version} (flat) items files are not read by the engine; save them once in Godot (version 2)."));
        return found;
    }
    for item in list {
        let Some(d) = item.as_object() else {
            errors.push(format!("{at}: an entry is not an object; it is skipped."));
            continue;
        };
        let fam_id = sget(d, "family", "");
        match families.iter().find(|f| !fam_id.is_empty() && sget(f, "id", "") == fam_id) {
            Some(family) => found.push(content("items", &resolve_item(d, family))),
            None => {
                errors.push(format!("{at}: item '{}' names the unknown family '{fam_id}'; it is kept as stored.", sget(d, "id", "")));
                found.push(content("items", d));
            }
        }
    }
    found
}

/// ItemModifiers.resolve: the flat row of a stored entry (normalized by the caller).
pub(crate) fn resolve_item(entry: &Dict, family: &Dict) -> Dict {
    let mut row = entry.clone();
    // U3.2-S1: a stored retired key and adjust modifiers on a retired number are ignored (and so not written back)
    for k in RETIRED_NUMBERS {
        row.shift_remove(k);
    }
    if let Some(Value::Array(list)) = row.get_mut("modifiers") {
        list.retain(|m| !is_retired_adjust(m));
    }
    let t = item_type_of(family);
    let family_id = sget(family, "id", "");
    row.insert("family".into(), Value::from(family_id.clone()));
    row.insert("item_type".into(), Value::from(t.clone()));
    row.insert("weapon_type".into(), Value::from(if GEAR_TYPES.contains(&t.as_str()) { family_id } else { String::new() }));
    let hands = hands_of(family);
    let hand = if hands.is_empty() {
        String::new()
    } else {
        let h = sget(entry, "hand", "");
        if h.is_empty() {
            default_hand(family)
        } else {
            h
        }
    };
    row.insert("hand".into(), Value::from(hand.clone()));
    if !hands.is_empty() {
        row.insert("slot".into(), Value::from(if hand == "off" { "OFF_HAND" } else { "MAIN_HAND" }));
    } else {
        let slots = json::strings(json::get(family, "slots"));
        if sget(entry, "slot", "").is_empty() && !slots.is_empty() {
            row.insert("slot".into(), Value::from(slots[0].clone()));
        }
    }
    row.insert("two_handed".into(), Value::Bool(hand == "main_2h"));
    row.insert("off_hand_category".into(), Value::from(off_hand_category(family)));
    if t == "WEAPON" {
        let scaling = json::strings(json::get(family, "scaling_stats"));
        if sget(entry, "scaling_stat", "").is_empty() {
            row.insert("scaling_stat".into(), Value::from(scaling.first().cloned().unwrap_or_else(|| "STR".into())));
        }
    } else {
        row.shift_remove("scaling_stat");
    }
    // ItemFamilies.base_of + the adjust deltas (raw_numbers)
    let base_all = json::dget(family, "base");
    let key = if hands.is_empty() { "none".to_string() } else if hands.contains(&hand) { hand.clone() } else { default_hand(family) };
    let base = json::dget(base_all, &key);
    let mods: Vec<Dict> = json::aget(entry, "modifiers").iter().map(|m| m.as_object().cloned().unwrap_or_default()).collect();
    for (field, flat, min) in NUMBER_FIELDS {
        let mut n = to_int_or(base.get(field).unwrap_or(&int(min)), min);
        for m in mods.iter().filter(|m| sget(m, "kind", "") == "adjust" && sget(m, "field", "") == field) {
            n += json::to_int(json::get(m, "delta"));
        }
        row.insert(flat.into(), int(n));
    }
    let (mut reqs, mut bonuses, mut effects) = (Dict::new(), Dict::new(), Vec::new());
    for m in &mods {
        match sget(m, "kind", "").as_str() {
            "requirement" => {
                reqs.insert(sget(m, "stat", ""), int(json::to_int(json::get(m, "value"))));
            }
            "stat_bonus" => {
                bonuses.insert(sget(m, "stat", ""), int(json::to_int(json::get(m, "value"))));
            }
            "effect" => effects.push(Value::Object(json::dget(m, "effect").clone())),
            _ => {}
        }
    }
    row.insert("required_stats".into(), Value::Object(reqs));
    row.insert("stat_bonuses".into(), Value::Object(bonuses));
    row.insert("effects".into(), Value::Array(effects));
    let tiers: Vec<Value> = json::aget(entry, "tiers").iter().map(|t| Value::Object(legacy_tier(&t.as_object().cloned().unwrap_or_default()))).collect();
    row.insert("threshold_tiers".into(), Value::Array(tiers));
    if sget(entry, "damage_type", "").is_empty() {
        row.insert("damage_type".into(), Value::from(sget(family, "damage_type", "")));
    }
    // U3.2-S2 (RULESET1 K1 for `reach`): the family's engine rules ride on the resolved row, so the gear rules read them
    // and never a family id; the writer leaves them out of the stored form (`store::ITEM_DERIVED`)
    row.shift_remove(rpgkit_data::ITEM_REACH_RULE);
    let rules = json::strings(json::get(family, "engine_rules"));
    row.insert(rpgkit_data::ITEM_FAMILY_RULES.into(), Value::Array(rules.into_iter().map(Value::from).collect()));
    // U3.2-S4: the family's Level-2 bonus and Level-3 mastery passive ride on the row the same way (left out of the
    // stored form; an empty one is no key, so a row of a family without them reads as before)
    let level2 = json::dget(family, "level2_bonus");
    if !level2.is_empty() {
        row.insert(rpgkit_data::ITEM_FAMILY_LEVEL2.into(), Value::Object(level2.clone()));
    }
    let level3 = sget(family, "level3_passive", "");
    if !level3.is_empty() {
        row.insert(rpgkit_data::ITEM_FAMILY_LEVEL3.into(), Value::from(level3));
    }
    row
}

/// One row of ItemModifiers.legacy_tiers.
pub(crate) fn legacy_tier(tier: &Dict) -> Dict {
    let when = json::dget(tier, "when").clone();
    let is_stat = sget(&when, "type", "") == "stat_at_least";
    let bonus = json::dget(tier, "bonus").clone();
    let desc = sget(tier, "desc", "");
    let mut row = Dict::new();
    row.insert("stat".into(), Value::from(if is_stat { sget(&when, "stat", "STR") } else { String::new() }));
    row.insert("threshold".into(), int(if is_stat { json::to_int(json::get(&when, "value")) } else { 0 }));
    row.insert("bonus_power".into(), int(json::to_int(json::get(&bonus, "power"))));
    row.insert("desc".into(), Value::from(if desc.is_empty() { tier_desc(tier) } else { desc }));
    if !is_stat || bonus.keys().any(|k| k != "power") {
        row.insert("when".into(), Value::Object(when));
        row.insert("bonus".into(), Value::Object(bonus));
    }
    row
}

/// ItemText.render: an item description template filled from the resolved row ({name}, {family}, {power}, {range},
/// {hand}, {burden}, {mitigation}, {req:STAT}, {bon:STAT}, {tier<n>.value}, {tier<n>.power}); a token not
/// valid for the item type renders as nothing; a description without tokens is returned as it is.
pub fn render_item(template: &str, row: &Dict) -> String {
    if !template.contains('{') && !template.contains('}') {
        return template.to_string();
    }
    let item_type = sget(row, "item_type", "WEAPON");
    let equip = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"].contains(&item_type.as_str());
    let weapon = item_type == "WEAPON";
    let tiers = json::aget(row, "tiers");
    render_tokens(template, |token, suffix| {
        let number = |k: &str| Some(json::to_int(json::get(row, k)).to_string());
        if let Some((n, part)) = token.strip_prefix("tier").and_then(|t| t.split_once('.')).filter(|(n, _)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())).and_then(|(n, p)| Some((n.parse::<usize>().ok()?, p))) {
            if !equip || (part == "power" && !weapon) || !["value", "power"].contains(&part) {
                return None;
            }
            let tier = tiers.get(n.checked_sub(1)?)?.as_object()?;
            return Some(match part {
                "value" => {
                    let when = json::dget(tier, "when");
                    json::to_int(when.get("value").or_else(|| when.get("level")).unwrap_or(&Value::Null)).to_string()
                }
                _ => {
                    let p = json::to_int(json::get(json::dget(tier, "bonus"), "power"));
                    if p > 0 { p.to_string() } else { String::new() }
                }
            });
        }
        match token {
            "name" | "family" => Some(sget(row, token, "")),
            "power" if weapon => number("base_power"),
            "range" if weapon => number("base_range"),
            "hand" if weapon || item_type == "SHIELD" => Some(
                match sget(row, "hand", "").as_str() {
                    "" => "",
                    "main_1h" => "main hand, one-handed",
                    "main_2h" => "main hand, two-handed",
                    "off" => "off hand",
                    _ => "-",
                }
                .to_string(),
            ),
            "burden" | "mitigation" if equip => number(token),
            "req" | "bon" if equip => {
                let stats = json::dget(row, if token == "req" { "required_stats" } else { "stat_bonuses" });
                Some(stats.get(suffix).map(|v| json::to_int(v).to_string()).unwrap_or_default())
            }
            _ => None,
        }
    })
}

/// ItemText.tier_desc: "<label>: <bonus text>".
pub(crate) fn tier_desc(tier: &Dict) -> String {
    let label = sget(tier, "label", "").trim().to_string();
    let bonus = json::dget(tier, "bonus");
    let mut parts = Vec::new();
    for (key, name) in [("power", "Power"), ("mitigation", "Mitigation")] {
        let n = json::to_int(json::get(bonus, key));
        if n != 0 {
            parts.push(format!("+{n} {name}"));
        }
    }
    for (stat, v) in json::dget(bonus, "stat_bonuses") {
        parts.push(format!("+{} {stat}", json::to_int(v)));
    }
    let fx = json::aget(bonus, "effects").len();
    if fx > 0 {
        parts.push(format!("+{fx} effect{}", if fx == 1 { "" } else { "s" }));
    }
    let text = parts.join(", ");
    if label.is_empty() {
        text
    } else if text.is_empty() {
        label
    } else {
        format!("{label}: {text}")
    }
}

// ---- MapCatalog / QuestCatalog / ServiceCatalog / XpCurve ------------------------------------------------------

pub(crate) fn load_list(path: &Path, errors: &mut Vec<String>, normalize_one: fn(&Dict) -> Dict) -> Vec<Value> {
    let Some(doc) = read(path, errors) else { return Vec::new() };
    entries_of(doc).0.iter().filter_map(Value::as_object).map(|e| Value::Object(normalize_one(e))).collect()
}

/// The fixed keys first (raw values, JsonUtil.normalize'd), then the other keys except "_" ones.
fn template(entry: &Dict, keys: &[(&str, Value)]) -> Dict {
    let e = json::normalize_dict(entry);
    let mut out = Dict::new();
    for (k, fallback) in keys {
        let v = if *k == "id" { Value::from(sget(&e, "id", "")) } else { e.get(*k).cloned().unwrap_or_else(|| fallback.clone()) };
        out.insert(k.to_string(), v);
    }
    for (k, v) in &e {
        if !out.contains_key(k) && !k.starts_with('_') {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

/// MapRules.normalize_template; PROPS1 1: `props` (after `paths`), each prop with its defaults (`prop_full`);
/// REGIONGROUND1 KT 1.2 and SC: `ground`, `ground_legend`, `ground_patches` (each patch with its default strength,
/// `ground_patch_full`) and `wall_storeys` (after `props`). `rows` alone decide walking: nothing in the world or battle
/// crates reads these four.
pub(crate) fn map_template(e: &Dict) -> Dict {
    let (s, a, d) = (Value::from(""), Value::Array(Vec::new()), Value::Object(Map::new()));
    let mut t = template(e, &[("id", s.clone()), ("name", s), ("rows", a.clone()), ("zones", a.clone()), ("zone_legend", d.clone()), ("anchors", a.clone()), ("paths", d.clone()), ("props", a.clone()), ("ground", a.clone()), ("ground_legend", d), ("ground_patches", a.clone()), ("wall_storeys", a.clone()), ("tags", a)]);
    if let Some(Value::Array(list)) = t.get("props") {
        let full = list.iter().map(|p| p.as_object().map_or_else(|| p.clone(), |o| Value::Object(prop_full(o)))).collect();
        t.insert("props".into(), Value::Array(full));
    }
    if let Some(Value::Array(list)) = t.get("ground_patches") {
        let full = list.iter().map(|p| p.as_object().map_or_else(|| p.clone(), |o| Value::Object(ground_patch_full(o)))).collect();
        t.insert("ground_patches".into(), Value::Array(full));
    }
    t
}

/// A placed ground patch's keys in stored order (KT 1.2).
pub const GROUND_PATCH_KEYS: [&str; 6] = ["id", "x", "y", "w", "h", "strength"];
/// KT 1.2: a placed patch's default strength.
pub const GROUND_PATCH_STRENGTH: f64 = 0.12;

/// A placed ground patch with its default strength in `GROUND_PATCH_KEYS` order; the other keys stay missing when
/// missing and unknown keys follow (the checks name both).
pub fn ground_patch_full(p: &Dict) -> Dict {
    let p = json::normalize_dict(p);
    let mut out = Dict::new();
    for k in GROUND_PATCH_KEYS {
        match p.get(k) {
            Some(v) => {
                out.insert(k.to_string(), v.clone());
            }
            None if k == "strength" => {
                out.insert(k.to_string(), Value::from(GROUND_PATCH_STRENGTH));
            }
            None => {}
        }
    }
    for (k, v) in &p {
        if !out.contains_key(k) {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

/// A map prop's keys in stored order (PROPS1 1).
pub const PROP_KEYS: [&str; 9] = ["id", "art", "x", "y", "w", "h", "layer", "variant", "flip"];

/// A map prop with its defaults (`w` 1, `h` 1, `layer` sort, `variant` -1, `flip` false) in `PROP_KEYS` order; the
/// required keys (`id`, `art`, `x`, `y`) stay missing when missing and unknown keys follow (the checks name both).
pub fn prop_full(p: &Dict) -> Dict {
    let p = json::normalize_dict(p);
    let fallback = |k: &str| match k {
        "w" | "h" => Some(int(1)),
        "layer" => Some(Value::from("sort")),
        "variant" => Some(int(-1)),
        "flip" => Some(Value::Bool(false)),
        _ => None,
    };
    let mut out = Dict::new();
    for k in PROP_KEYS {
        if let Some(v) = p.get(k).cloned().or_else(|| fallback(k)) {
            out.insert(k.to_string(), v);
        }
    }
    for (k, v) in &p {
        if !out.contains_key(k) {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

/// QuestCatalog.normalize.
pub(crate) fn quest_template(e: &Dict) -> Dict {
    let (s, a, d) = (Value::from(""), Value::Array(Vec::new()), Value::Object(Map::new()));
    template(e, &[("id", s.clone()), ("name", s.clone()), ("description", s), ("objectives", a.clone()), ("rewards", d), ("tags", a)])
}

/// ServiceCatalog.normalize.
pub(crate) fn service(e: &Dict) -> Dict {
    let s = Value::from("");
    template(e, &[("id", s.clone()), ("text", s)])
}

/// XpCurve.load_file: the file as parsed when it validates, else the defaults.
fn xp_curve(path: &Path, errors: &mut Vec<String>) -> Dict {
    let doc = read(path, errors).and_then(|v| v.as_object().cloned());
    let problems = match &doc {
        None => vec![format!("XP curve file '{}' not found or not valid JSON; using defaults.", path.display())],
        Some(d) => xp_problems(d),
    };
    if problems.is_empty() {
        return doc.unwrap();
    }
    errors.extend(problems);
    let defaults = serde_json::json!({"version": 1, "xp_to_next": [40, 70, 110, 160, 220, 290, 370, 460, 560],
        "kill_xp": {"base": 10, "per_level": 5, "tier_mult": {"minion": 0.5, "standard": 1.0, "elite": 2.0, "boss": 5.0, "hero": 1.0},
        "level_delta_step": 0.15, "level_delta_min": 0.25, "level_delta_max": 2.0}});
    defaults.as_object().cloned().unwrap()
}

/// XpCurve.validate.
fn xp_problems(doc: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let steps = json::aget(doc, "xp_to_next");
    let want = rpgkit_rules::stats::MAX_LEVEL as usize - 1;
    if steps.len() != want {
        errors.push(format!("xp_to_next needs exactly {want} entries (got {}).", steps.len()));
    }
    if steps.iter().any(|s| !s.is_number() || json::to_int(s) <= 0) {
        errors.push("xp_to_next entries must be positive numbers.".into());
    }
    let kill = json::dget(doc, "kill_xp");
    if kill.is_empty() {
        errors.push("kill_xp is missing.".into());
        return errors;
    }
    for key in ["base", "per_level", "level_delta_step", "level_delta_min", "level_delta_max"] {
        if !json::get(kill, key).is_number() {
            errors.push(format!("kill_xp.{key} must be a number."));
        }
    }
    let mult = json::dget(kill, "tier_mult");
    let tiers = ["minion", "standard", "elite", "boss", "hero"];
    for t in tiers {
        if !mult.contains_key(t) {
            errors.push(format!("kill_xp.tier_mult is missing tier '{t}'."));
        }
    }
    for (k, v) in mult {
        if !tiers.contains(&k.as_str()) {
            errors.push(format!("kill_xp.tier_mult has unknown tier '{k}'."));
        } else if to_float_or(v, 0.0) <= 0.0 {
            errors.push(format!("kill_xp.tier_mult.{k} must be > 0."));
        }
    }
    if errors.is_empty() {
        let (lo, hi) = (json::f(kill, "level_delta_min", 0.0), json::f(kill, "level_delta_max", 0.0));
        if lo > 1.0 || hi < 1.0 {
            errors.push("level_delta_min must be <= 1 <= level_delta_max.".into());
        }
        if lo <= 0.0 {
            errors.push("level_delta_min must be > 0.".into());
        }
    }
    errors
}
