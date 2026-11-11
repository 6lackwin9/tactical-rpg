//! Recipes (PK2-S3, `docs/sessions/PK2_GRAFTS.md` section 4; engine-only, no Godot original): the checks of one row of
//! the kind `recipes` against the catalog loaded before it (the id, name, origin and target, the cost's purse amounts
//! and items, the effects: known, their keys, their target, their refs to items, body slots, unit tracks and class
//! roles, and the effects whose system is not built yet) and across rows (unique ids). The loader keeps the rows that
//! pass, in file order. The world refs (the offering character, the `requires` conditions) are checked over the
//! worlds (`world_errors`, the whole-catalog report).

use super::View;
use crate::source::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::recipe::{self as r, RECIPE_COST_KEYS, RECIPE_COUNT_MAX, RECIPE_DELTA, RECIPE_EFFECTS, RECIPE_EFFECTS_MAX, RECIPE_PCT, RECIPE_TARGETS};
use serde_json::Value;

/// The catalog kind (file data/catalog/recipes.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::RECIPES_KIND;
/// The id's length range, the name's limit, a doctrine axis id's limit, a purse amount's top.
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
pub const NAME_MAX: usize = 40;
pub const AXIS_MAX: usize = 24;
pub const PURSE_MAX: i64 = 9_999_999;

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// The schema options of a field of `kind`.
fn options(kind: &str, key: &str) -> Vec<String> {
    fields("content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// The row `id` of `kind` in `loaded` ({kind: [row]}), if any.
fn row<'a>(loaded: &'a Dict, kind: &str, id: &str) -> Option<&'a Dict> {
    json::arr(json::get(loaded, kind)).iter().map(json::dict).find(|e| json::s(e, "id") == id)
}

fn whole_in(v: Option<&Value>, lo: i64, hi: i64) -> bool {
    v.is_some_and(|v| json::is_int(v) && (lo..=hi).contains(&json::to_int(v)))
}

/// A non-zero whole delta in `range` at `key`.
fn delta_errors(fx: &Dict, key: &str, range: (i64, i64), at: &str, out: &mut Vec<String>) {
    if !whole_in(fx.get(key), range.0, range.1) || json::to_int(json::get(fx, key)) == 0 {
        out.push(format!("{at}: {key} must be a whole number {}..{}, not 0.", range.0, range.1));
    }
}

/// The problems of each row (same order as the rows), on its own against `loaded` (the catalog kinds loaded before:
/// items, body slots, unit tracks) and against the others (unique ids).
pub fn check_rows(rows: &[Dict], loaded: &Dict) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = rows.iter().map(|e| row_errors(e, loaded)).collect();
    for (i, found) in out.iter_mut().enumerate() {
        let id = json::s(&rows[i], "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another recipe."));
        }
    }
    out
}

/// The own problems of one (normalized) row against `loaded`.
pub fn row_errors(e: &Dict, loaded: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let id = json::s(e, "id");
    if !json::id_ok(&id) || !(ID_MIN..=ID_MAX).contains(&chars(&id)) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _."));
    }
    let n = chars(&json::s(e, "name"));
    if n == 0 {
        out.push("A recipe needs a name.".into());
    } else if n > NAME_MAX {
        out.push(format!("Name is {n} characters (max {NAME_MAX})."));
    }
    if json::s(e, "npc").is_empty() {
        out.push("A recipe needs the character who offers it (npc).".into());
    }
    let origin = json::s(e, "origin");
    let allowed = options(KIND, "origin");
    if !allowed.contains(&origin) {
        out.push(format!("Origin: '{origin}' is not one of {}.", allowed.join(", ")));
    }
    let target = json::s(e, "target");
    if !RECIPE_TARGETS.contains(&target.as_str()) {
        out.push(format!("Target: '{target}' is not one of {}.", RECIPE_TARGETS.join(", ")));
    }
    if !json::get(e, "requires").is_array() {
        out.push("Requires must be a list of conditions.".into());
    }
    out.extend(cost_errors(json::get(e, "cost"), loaded));
    out.extend(effects_errors(json::get(e, "effects"), &target, loaded));
    out
}

/// The cost: an object of the purse amounts (whole, 0..PURSE_MAX) and `items` {known item: 1..RECIPE_COUNT_MAX}.
fn cost_errors(raw: &Value, loaded: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let Some(cost) = raw.as_object() else { return vec!["Cost must be an object {gold, supplies, items}.".into()] };
    for key in cost.keys().filter(|k| !RECIPE_COST_KEYS.contains(&k.as_str())) {
        out.push(format!("Cost: unknown key '{key}' (use {}).", RECIPE_COST_KEYS.join(", ")));
    }
    for key in ["gold", "supplies"] {
        if cost.contains_key(key) && !whole_in(cost.get(key), 0, PURSE_MAX) {
            out.push(format!("Cost {key} must be a whole number 0..{PURSE_MAX}."));
        }
    }
    match cost.get("items") {
        None => {}
        Some(Value::Object(items)) => {
            for (item, n) in items {
                if row(loaded, "items", item).is_none() {
                    out.push(format!("Cost items: unknown item '{item}'."));
                }
                if !whole_in(Some(n), 1, RECIPE_COUNT_MAX) {
                    out.push(format!("Cost items: {item} count must be a whole number 1..{RECIPE_COUNT_MAX}."));
                }
            }
        }
        Some(_) => out.push("Cost items must be an object {item id: count}.".into()),
    }
    out
}

/// The effects: 1..RECIPE_EFFECTS_MAX known effect rows, each with its own keys, fitting the recipe's target, its refs
/// known, and none whose system is not built yet.
fn effects_errors(raw: &Value, target: &str, loaded: &Dict) -> Vec<String> {
    let Some(list) = raw.as_array().filter(|a| !a.is_empty() && a.len() <= RECIPE_EFFECTS_MAX) else {
        return vec![format!("Effects must be a list of 1-{RECIPE_EFFECTS_MAX} effects.")];
    };
    let mut out = Vec::new();
    for (i, raw) in list.iter().enumerate() {
        let Some(fx) = raw.as_object() else {
            out.push(format!("Effect {} must be an object {{\"effect\", ...}}.", i + 1));
            continue;
        };
        let name = json::s(fx, "effect");
        let at = format!("Effect {} ({name})", i + 1);
        let Some(keys) = r::effect_keys(&name) else {
            out.push(format!("Effect {}: unknown effect '{name}' (use {}).", i + 1, RECIPE_EFFECTS.iter().map(|e| e.0).collect::<Vec<_>>().join(", ")));
            continue;
        };
        for key in fx.keys().filter(|k| *k != "effect" && !keys.contains(&k.as_str())) {
            out.push(format!("{at}: unknown key '{key}'."));
        }
        let wants = r::effect_target(&name);
        if !wants.is_empty() && wants != target {
            out.push(format!("{at}: works on a {wants}, but the recipe's target is {target}."));
        }
        if let Some(system) = r::pending_system(&name) {
            out.push(format!("{at}: needs {system}, which this build does not have yet."));
            continue;
        }
        out.extend(effect_errors(&name, fx, &at, loaded));
    }
    out
}

/// One built effect's own problems.
fn effect_errors(name: &str, fx: &Dict, at: &str, loaded: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let slot = |out: &mut Vec<String>| -> Option<String> {
        let id = json::s(fx, "slot");
        match row(loaded, rpgkit_data::BODY_SLOTS_KIND, &id) {
            Some(s) => Some(json::s(s, "role")),
            None => {
                out.push(format!("{at}: unknown body slot '{id}'."));
                None
            }
        }
    };
    match name {
        r::FX_INSTALL => {
            let role = slot(&mut out);
            let item = json::s(fx, "item");
            match row(loaded, "items", &item) {
                None => out.push(format!("{at}: unknown item '{item}'.")),
                Some(i) if json::s(i, "item_type") != rpgkit_data::GRAFT => out.push(format!("{at}: '{item}' is not a graft.")),
                Some(i) => {
                    let fits = json::s_or(i, "body_role", rpgkit_data::BODY_ROLE_MONSTER);
                    if role.is_some_and(|r| r != fits) {
                        out.push(format!("{at}: body slot '{}' does not take {fits} grafts.", json::s(fx, "slot")));
                    }
                }
            }
        }
        r::FX_REMOVE => {
            slot(&mut out);
        }
        r::FX_TRACK => {
            let track = json::s(fx, "track");
            if row(loaded, rpgkit_data::UNIT_TRACKS_KIND, &track).is_none() {
                out.push(format!("{at}: unknown unit track '{track}'."));
            }
            delta_errors(fx, "delta", RECIPE_DELTA, at, &mut out);
        }
        r::FX_MAX_HP_PCT => delta_errors(fx, "pct", RECIPE_PCT, at, &mut out),
        r::FX_FLOOR_OFFSET => delta_errors(fx, "delta", RECIPE_DELTA, at, &mut out),
        r::FX_DOCTRINE => {
            let axis = json::s(fx, "axis");
            if !json::id_ok(&axis) || axis.is_empty() || chars(&axis) > AXIS_MAX {
                out.push(format!("{at}: axis '{axis}' must be an id of 1-{AXIS_MAX} characters (a-z, 0-9 and _)."));
            }
            delta_errors(fx, "delta", RECIPE_DELTA, at, &mut out);
        }
        r::FX_TRAIN => {
            let role = json::s(fx, "role");
            if !rpgkit_data::PROF_ROLES.contains(&role.as_str()) {
                out.push(format!("{at}: role '{role}' is not one of {}.", rpgkit_data::PROF_ROLES.join(", ")));
            }
            let top = i64::from(rpgkit_rules::stats::PROF_MAX_LEVEL);
            if !whole_in(fx.get("to"), 1, top) {
                out.push(format!("{at}: to must be the level reached, a whole number 1..{top}."));
            }
        }
        r::FX_SALVAGE => out.extend(salvage_errors(fx, at, loaded)),
        r::FX_DRILL => {
            // PK2-S4: the mastery XP before the member's proficiency rate
            let (lo, hi) = rpgkit_data::mastery::MASTERY_XP_RANGE;
            if !whole_in(fx.get("xp"), lo, hi) {
                out.push(format!("{at}: xp must be a whole number {lo}..{hi}."));
            }
        }
        _ => {}
    }
    out
}

/// A salvage effect: `gold` (whole, 0..PURSE_MAX), `tiers` {item rarity: known item}, `themes` {theme id: known item};
/// it gives something.
fn salvage_errors(fx: &Dict, at: &str, loaded: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    if fx.contains_key("gold") && !whole_in(fx.get("gold"), 0, PURSE_MAX) {
        out.push(format!("{at}: gold must be a whole number 0..{PURSE_MAX}."));
    }
    let rarities = options("items", "rarity");
    for (key, keyed) in [("tiers", true), ("themes", false)] {
        match fx.get(key) {
            None => {}
            Some(Value::Object(m)) => {
                for (k, item) in m {
                    if keyed && !rarities.contains(k) {
                        out.push(format!("{at}: {key} key '{k}' is not one of {}.", rarities.join(", ")));
                    }
                    if !keyed && (!json::id_ok(k) || k.is_empty()) {
                        out.push(format!("{at}: {key} key '{k}' must be a theme id (a-z, 0-9 and _)."));
                    }
                    if row(loaded, "items", &gstr(item)).is_none() {
                        out.push(format!("{at}: {key} {k}: unknown item '{}'.", gstr(item)));
                    }
                }
            }
            Some(_) => out.push(format!("{at}: {key} must be an object.")),
        }
    }
    let gives = json::i(fx, "gold", 0) > 0 || ["tiers", "themes"].iter().any(|k| !json::dget(fx, k).is_empty());
    if !gives {
        out.push(format!("{at}: a salvage gives gold, tiers or themes."));
    }
    out
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems against the
/// loaded catalog, those against the loaded rows (its old row taken out), and its world refs.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    let mut found = check_rows(&rows, &loaded_of(v)).pop().unwrap_or_default();
    found.extend(world_errors(v, e));
    found
}

/// The catalog kinds a recipe names, as the loader holds them ({kind: [row]}).
fn loaded_of(v: &View) -> Dict {
    ["items", rpgkit_data::BODY_SLOTS_KIND, rpgkit_data::UNIT_TRACKS_KIND]
        .iter()
        .map(|k| (k.to_string(), Value::Array(v.entries(k).iter().cloned().map(Value::Object).collect())))
        .collect()
}

/// The world refs of a row: the offering character resolves in some world (the first that has it), and the `requires`
/// conditions check in that world (the dialogue grammar).
pub fn world_errors(v: &View, e: &Dict) -> Vec<String> {
    let npc = json::s(e, "npc");
    if npc.is_empty() {
        return Vec::new();
    }
    let Some(w) = v.worlds.world_ids().into_iter().find(|w| v.worlds.has_entry(w, "characters", &npc)) else {
        return vec![format!("Offered by: unknown character '{npc}'.")];
    };
    if !json::get(e, "requires").is_array() {
        return Vec::new();
    }
    super::dialogue::conditions_errors(v, &w, json::get(e, "requires"), "Requires")
}
