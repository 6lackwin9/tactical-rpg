//! The content checks of the Forge (PORT-CHECKS; Godot's ContentCatalog.validate_entry / validate_all / warnings_for /
//! warnings_all and the validators under scripts/content/validators): hard errors that block a save and soft warnings,
//! per entry and over the whole catalog, message for message. They read the loaded catalog (`source::load`: flat skill
//! rows, resolved items, derived item families).

pub mod assets;
pub mod dialogue;
pub mod encounter;
pub mod item;
pub mod map;
pub mod quest;
pub mod reward;
pub mod shop;
pub mod simple;
pub mod skill;
pub mod travel;
pub mod unit;
pub mod world;

/// The catalog kinds the checks cover so far.
pub const PORTED: [&str; 8] = ["skills", "terrain", "passives", "classes", "units", "encounters", "items", "item_families"];

use crate::source::{content, fields};
use aegis_content::schema::is_id;
use aegis_core::json::{self, gstr, Dict};
use indexmap::IndexMap;
use serde_json::Value;

/// The catalog the checks read: {kind: [entry]} as `source::load` gives it, with the game tables beside it
/// ("item_families", "quest_templates", "maps", "services").
pub struct View {
    rows: IndexMap<String, Vec<Dict>>,
    /// The asset layers of the checked data's game (ASSETORG2: the game's, its kit's, the engine's; empty: the asset
    /// checks are skipped).
    pub assets: aegis_core::assets::Layers,
    /// The same catalog typed (classes, the skill families: ClassArchetype, SkillCatalog).
    pub typed: emberwake_data::Catalog,
    /// The worlds over the catalog (WorldCatalog.resolve: a catalog encounter's qualified purse loot).
    pub worlds: aegis_content::world::Catalogs,
    /// True once the catalog has a terrain file (missing terrain rows are errors then, else warnings).
    pub terrain_file: bool,
    /// What the loader reported, catalog and worlds (Godot's load errors; validate_all lists them under "load").
    pub load_errors: Vec<String>,
}

/// ContentSchema.NO_ITEM: a unit gear slot or pouch left empty on purpose.
pub const NO_ITEM: &str = "none";
pub const LOADOUT_SLOTS: [&str; 6] = ["main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2"];
pub const ORIGIN_SYSTEM: &str = "system";

impl View {
    /// From the loader's catalog, worlds and game values (the families, quest templates, maps and services sit with
    /// the game data).
    pub fn new(catalog: &Value, worlds: &Value, game: &Value, assets: aegis_core::assets::Layers) -> Self {
        let mut rows: IndexMap<String, Vec<Dict>> = IndexMap::new();
        for (kind, list) in catalog.as_object().into_iter().flatten() {
            rows.insert(kind.clone(), json::arr(list).iter().map(|e| json::dict(e).clone()).collect());
        }
        for (name, key) in [("item_families", "item_families"), ("quest_templates", "quests"), ("maps", "maps"), ("services", "services")] {
            rows.insert(name.into(), json::arr(&game[key]).iter().map(|e| json::dict(e).clone()).collect());
        }
        Self { rows, assets, typed: emberwake_data::Catalog::from_value(catalog), worlds: aegis_content::world::Catalogs::from_values(catalog, worlds), terrain_file: true, load_errors: Vec::new() }
    }

    pub fn entries(&self, kind: &str) -> &[Dict] {
        self.rows.get(kind).map_or(&[], |v| v.as_slice())
    }

    /// The entry `id` of `kind` (an empty one when there is none).
    pub fn get(&self, kind: &str, id: &str) -> &Dict {
        self.entries(kind).iter().find(|e| json::s(e, "id") == id).unwrap_or_else(|| json::empty())
    }

    pub fn has(&self, kind: &str, id: &str) -> bool {
        self.entries(kind).iter().any(|e| json::s(e, "id") == id)
    }
}

/// Whether a field's enum value is checked against its options: not the body fields (ASSETORG2: A1 checks `art`
/// against the asset layers with its own message).
pub fn enum_checked(f: &Dict) -> bool {
    !matches!(json::s(f, "options_from").as_str(), "body" | "body_inherit")
}

/// ContentSchema.options_of: a field's options, or the list its `options_from` names.
pub fn options_of(v: &View, f: &Dict) -> Vec<String> {
    // ASSETORG2: the bodies across the asset layers
    let bodies = |inherit: bool| {
        let mut out: Vec<String> = if inherit { vec![String::new()] } else { Vec::new() };
        out.extend(v.assets.bodies());
        out
    };
    let s = |l: &[&str]| l.iter().map(|x| x.to_string()).collect::<Vec<String>>();
    match json::s(f, "options_from").as_str() {
        "zones" => s(&emberwake_battle::grid::ZONE_IDS),
        "body" => bodies(false),
        "body_inherit" => bodies(true),
        "status" => std::iter::once(String::new()).chain(emberwake_rules::status::debuffs().into_iter().map(String::from)).collect(),
        "status_buff" => std::iter::once(String::new()).chain(emberwake_rules::status::ROWS.iter().filter(|r| r.buff && !r.grant.is_empty()).map(|r| r.id.to_string())).collect(),
        "weapon_types" => skill::GEAR_TYPES.iter().map(|g| g.to_string()).chain(std::iter::once("unarmed".to_string())).collect(),
        "debuffs_or_all" => emberwake_rules::status::debuffs().into_iter().map(String::from).chain(std::iter::once("all".to_string())).collect(),
        _ => json::aget(f, "options").iter().map(gstr).collect(),
    }
}

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// Godot's String.capitalize of a snake_case key: "main_hand" -> "Main Hand", "accessory_1" -> "Accessory 1".
pub(crate) fn capitalize(key: &str) -> String {
    key.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// ContentSchema.field_errors: id, name, enum options, text lengths.
pub fn field_errors(v: &View, kind: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let id = json::s(e, "id");
    if id.is_empty() {
        errors.push("ID is required.".into());
    } else if !is_id(&id) {
        errors.push(format!("ID '{id}' must be lowercase snake_case (a-z, 0-9, _)."));
    }
    if gstr(json::get(e, "name")).trim().is_empty() {
        errors.push("Name is required.".into());
    }
    for f in fields("content", kind).iter().map(json::dict) {
        let key = json::s(f, "key");
        let label = json::s(f, "label");
        let text = gstr(json::get(e, &key));
        if json::s(f, "type") == "enum" && visible(f, e) && enum_checked(f) {
            let options = options_of(v, f);
            if !options.contains(&text) {
                errors.push(format!("{label}: '{text}' is not one of {}.", options.join(", ")));
            }
        }
        if let Some(max) = f.get("max_len") {
            if chars(&text) > json::to_int(max) as usize {
                errors.push(format!("{label} is {} characters (max {}).", chars(&text), json::to_int(max)));
            }
        }
        if let Some(min) = f.get("min_len") {
            if !text.is_empty() && chars(&text) < json::to_int(min) as usize {
                errors.push(format!("{label} is {} characters (min {}).", chars(&text), json::to_int(min)));
            }
        }
    }
    errors
}

/// "skills" -> "skill" (Godot's trim_suffix("s"): "classes" -> "classe").
fn singular(kind: &str) -> &str {
    kind.strip_suffix('s').unwrap_or(kind)
}

/// ContentSchema.is_visible: a field's `when` condition on the entry.
pub fn visible(f: &Dict, e: &Dict) -> bool {
    let cond = json::dget(f, "when");
    if cond.is_empty() {
        return true;
    }
    let value = gstr(json::get(e, &json::s(cond, "key")));
    if let Some(allowed) = cond.get("in").and_then(Value::as_array) {
        return allowed.iter().any(|a| gstr(a) == value);
    }
    if let Some(denied) = cond.get("not_in").and_then(Value::as_array) {
        return !denied.iter().any(|a| gstr(a) == value);
    }
    true
}

/// ContentSchema.passes_filter: a reference target matches the field's `ref_filter`.
pub(crate) fn passes_filter(f: &Dict, target: &Dict) -> bool {
    let filter = json::dget(f, "ref_filter");
    filter.iter().all(|(k, want)| {
        let have = json::get(target, k);
        match want {
            Value::Array(wants) => wants.iter().any(|w| json::deep_equal(w, have)),
            _ => gstr(want) == gstr(have),
        }
    })
}

/// A gear slot or the pouch of a class / unit (the fields that may carry NO_ITEM).
fn is_gear_field(kind: &str, key: &str) -> bool {
    (kind == "units" || kind == "classes") && (LOADOUT_SLOTS.contains(&key) || key == "pouch")
}

/// ContentCatalog.validate_entry: the hard errors of `entry` (any shape the catalog stores) as entry `current_id`
/// ("" = a new entry).
pub fn validate_entry(v: &View, kind: &str, entry: &Dict, current_id: &str) -> Vec<String> {
    let entry = &crate::source::folded(kind, entry);
    let old_id = if current_id.is_empty() { json::s(entry, "id") } else { current_id.to_string() };
    let e = match kind {
        "items" => item::prepare(v, entry, v.get(kind, &old_id)),
        "item_families" => item::prepare_family(entry),
        _ => content(kind, entry),
    };
    let mut errors: Vec<String> = if kind != "terrain" { field_errors(v, kind, &e) } else { Vec::new() };
    let id = json::s(&e, "id");
    if id != current_id && v.has(kind, &id) {
        errors.push(format!("ID '{id}' is already used by another {} entry.", singular(kind)));
    }
    // PHASEFOLD 1.3: the arrival's units are checked as the retired wave_units field (after enemies) was.
    let arrival = if kind == "encounters" { encounter::arrival_units(&e) } else { None };
    for f in fields("content", kind) {
        let f = json::dict(f);
        errors.extend(ref_field_errors(v, kind, f, json::get(&e, &json::s(f, "key")), &e));
        if let (Some(units), "enemies") = (&arrival, json::s(f, "key").as_str()) {
            errors.extend(ref_field_errors(v, kind, encounter::wave_field(false), units, &e));
        }
    }
    let has_skill = |id: &str| v.has("skills", id);
    match kind {
        "skills" => errors.extend(skill::entry_errors(v, &e)),
        "passives" => errors.extend(simple::effect_errors(json::get(&e, "effects"), &has_skill, false, false)),
        "terrain" => errors.extend(simple::terrain_errors(&e)),
        "classes" => errors.extend(unit::class_errors(v, &e)),
        "units" => {
            errors.extend(unit::unit_errors(v, &e));
            errors.extend(assets::body_errors(v, &json::s(&e, "art")));
            errors.extend(reward::loot_errors(v, &e));
        }
        "encounters" => errors.extend(encounter::errors(v, &e)),
        "items" => errors.extend(item::item_errors(v, &e, item::family_of(v, &e), item::inherit(v))),
        "item_families" => errors.extend(item::family_errors(v, &e, current_id)),
        _ => {}
    }
    errors.extend(no_item_errors(kind, &e));
    errors
}

/// The reference problems of one ref field holding `value` (ContentCatalog.validate_entry's ref loop): listed twice,
/// unknown, filtered out.
fn ref_field_errors(v: &View, kind: &str, f: &Dict, value: &Value, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let ref_kind = json::s(f, "ref");
    if ref_kind.is_empty() || !visible(f, e) {
        return errors;
    }
    let key = json::s(f, "key");
    let label = json::s(f, "label");
    let mut targets: Vec<String> = Vec::new();
    if let Value::Array(list) = value {
        targets = list.iter().map(gstr).collect();
        if !json::b(f, "dupes", false) {
            let mut seen: Vec<&String> = Vec::new();
            for t in &targets {
                if seen.contains(&t) {
                    errors.push(format!("{label} lists '{t}' twice."));
                }
                seen.push(t);
            }
        }
    } else if !gstr(value).is_empty() {
        targets.push(gstr(value));
    }
    for t in targets {
        if t == NO_ITEM && is_gear_field(kind, &key) {
            continue;
        }
        let target = v.get(&ref_kind, &t);
        if target.is_empty() {
            errors.push(format!("{label}: unknown {} '{t}'.", singular(&ref_kind)));
        } else if !passes_filter(f, target) {
            errors.push(format!("{label}: '{t}' can't be used here."));
        }
    }
    errors
}

/// The NO_ITEM rules (BAL2-L 3.2): units only, alone in the pouch; no item may take the id.
fn no_item_errors(kind: &str, e: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    match kind {
        "items" if json::s(e, "id") == NO_ITEM => errors.push(format!("The item id '{NO_ITEM}' is reserved.")),
        "units" => {
            let pouch = json::sget(e, "pouch");
            if pouch.iter().any(|p| p == NO_ITEM) && pouch.len() > 1 {
                errors.push(format!("Battle pouch: '{NO_ITEM}' must be the only entry."));
            }
        }
        "classes" => {
            for f in fields("content", kind).iter().map(json::dict) {
                let key = json::s(f, "key");
                if !is_gear_field(kind, &key) {
                    continue;
                }
                let hit = if key == "pouch" { json::sget(e, &key).iter().any(|p| p == NO_ITEM) } else { json::s(e, &key) == NO_ITEM };
                if hit {
                    errors.push(format!("{}: '{NO_ITEM}' is only for units; leave the class slot empty.", json::s(f, "label")));
                }
            }
        }
        _ => {}
    }
    errors
}

/// ContentCatalog.warnings_for: the soft problems of one entry (never blocking).
pub fn warnings_for(v: &View, kind: &str, entry: &Dict) -> Vec<String> {
    let entry = &crate::source::folded(kind, entry);
    let e = content(kind, entry);
    match kind {
        "skills" => skill::warnings(entry, &e),
        "classes" => unit::class_warnings(v, &e),
        "units" => unit::unit_warnings(v, &e).into_iter().chain(reward::loot_warnings(v, &e)).chain(assets::portrait_and_palette_warnings(v, &Dict::new(), &e)).collect(),
        "encounters" => encounter::warnings(v, &e),
        "items" => item::item_warnings(v, &e),
        "item_families" => item::family_warnings(&crate::source::derive_family(&e)),
        _ => Vec::new(),
    }
}

/// ContentSchema.KINDS: the catalog kinds, in the order the whole-catalog reports walk them.
pub const KINDS: [&str; 8] = ["skills", "items", "item_families", "passives", "classes", "units", "encounters", "terrain"];

/// A whole-catalog report: {"kind:id" (or "kind:required", "skills:fold", "terrain:missing", "load"): [messages]}.
pub type Report = IndexMap<String, Vec<String>>;

/// ContentCatalog.validate_all: every entry's errors, the map templates' prop checks (PROPS1, `maps:<id>`), then the
/// rules across entries (required skills, one family per gear token, skill families that fold, a row per terrain
/// type) and the load problems.
pub fn validate_all(v: &View) -> Report {
    let mut problems = Report::new();
    for kind in KINDS {
        for e in v.entries(kind) {
            let id = json::s(e, "id");
            let errors = validate_entry(v, kind, e, &id);
            if !errors.is_empty() {
                problems.insert(format!("{kind}:{id}"), errors);
            }
        }
    }
    // PROPS1 2: the map templates' prop checks (engine-only; Godot's MapCatalog had no props)
    for t in v.entries("maps") {
        let errors = map::prop_errors(t);
        if !errors.is_empty() {
            problems.insert(format!("maps:{}", json::s(t, "id")), errors);
        }
    }
    let mut add = |key: &str, list: Vec<String>| {
        if !list.is_empty() {
            problems.entry(key.to_string()).or_default().extend(list);
        }
    };
    add("skills:required", skill::required_errors(v));
    add("item_families:required", item::family_set_errors(v.entries("item_families")));
    add("skills:fold", skill::collapse(v.entries("skills")).1);
    if v.terrain_file {
        add("terrain:missing", simple::missing_terrain(v.entries("terrain")).iter().map(|id| format!("No terrain row for '{id}'.")).collect());
    }
    add("load", v.load_errors.clone());
    problems
}

/// ContentCatalog.warnings_all: every entry's warnings (skills once per family, as ranked entries), and the terrain
/// rows the code defaults stand in for while there is no terrain file.
pub fn warnings_all(v: &View) -> Report {
    let mut out = Report::new();
    for kind in ["items", "item_families", "classes", "units", "encounters"] {
        for e in v.entries(kind) {
            let found = warnings_for(v, kind, e);
            if !found.is_empty() {
                out.insert(format!("{kind}:{}", json::s(e, "id")), found);
            }
        }
    }
    for ranked in skill::collapse(v.entries("skills")).0 {
        let found = warnings_for(v, "skills", &ranked);
        if !found.is_empty() {
            out.insert(format!("skills:{}", json::s(&ranked, "id")), found);
        }
    }
    if !v.terrain_file {
        let missing: Vec<String> = simple::missing_terrain(v.entries("terrain")).iter().map(|id| format!("No terrain row for '{id}': the code defaults apply.")).collect();
        if !missing.is_empty() {
            out.insert("terrain:missing".into(), missing);
        }
    }
    out
}
