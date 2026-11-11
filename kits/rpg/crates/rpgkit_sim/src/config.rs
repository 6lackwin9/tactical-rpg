//! PR5-33 (LAYER-UP-DATA): a game's sim description (`SimGame`) read from its JSON file, so the game's sim crate names
//! no world, flag, encounter, class or skill id. The shape (every key optional; a missing one is the `SimGame` default):
//!
//! ```json
//! {"default_world": "<world>", "feel_encounters": ["<encounter>"],
//!  "story": {"world", "branch_flag", "choice", "branches": [["<label>", <flag set>]], "usage", "extra_example"},
//!  "classes": {"ids": [], "default_comp": [], "trio_lead", "elite", "boss", "support_baseline",
//!   "roles": [{"class", "primary", "secondary": [], "loadouts": [{"role", "skills": []}],
//!              "l7_swaps": [{"role", "skill", "with"}]}],
//!   "generated": [{"id", "enemies": [["<class>", "<tier>"]], "wave": [...]}],
//!   "scenarios": [{"id", "role", "allies": [], "enemies": [...], "priority", "block": [x, y],
//!                  "tier_until": {"level", "slot", "tier"} | null}],
//!   "s2_enemies": [["<class>", "<tier>"]]}}
//! ```
//!
//! The class tables are `'static` (`SimClasses`), so the parsed text is leaked once: a game parses its file once per
//! process (the game's sim crate keeps it in a `OnceLock`).

use crate::roles::ClassRoles;
use crate::scenarios::{Generated, Roster, Scenario};
use crate::{SimClasses, SimGame, StoryRun};
use serde_json::Value;

/// The sim description in JSON text `text` (Err: the parse error or the first malformed key).
pub fn game_from_json(text: &str) -> Result<SimGame, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("sim config: {e}"))?;
    game_from_value(&v)
}

/// The sim description in `v` (the shape in the module docs).
pub fn game_from_value(v: &Value) -> Result<SimGame, String> {
    let o = v.as_object().ok_or("sim config: not an object")?;
    let story = match o.get("story") {
        None | Some(Value::Null) => None,
        Some(s) => Some(story(s)?),
    };
    Ok(SimGame {
        default_world: text(v, "default_world")?.to_string(),
        story,
        feel_encounters: owned_list(v, "feel_encounters")?,
        classes: match o.get("classes") {
            None | Some(Value::Null) => SimClasses::default(),
            Some(c) => classes(c)?,
        },
    })
}

fn story(v: &Value) -> Result<StoryRun, String> {
    let branches = list(v, "branches")?
        .iter()
        .map(|b| match b.as_array().map(Vec::as_slice) {
            Some([Value::String(label), Value::Bool(set)]) => Ok((label.clone(), *set)),
            _ => Err(format!("sim config: story branch {b} is not [label, bool]")),
        })
        .collect::<Result<_, _>>()?;
    Ok(StoryRun {
        world: text(v, "world")?.to_string(),
        branch_flag: text(v, "branch_flag")?.to_string(),
        choice: text(v, "choice")?.to_string(),
        branches,
        usage: text(v, "usage")?.to_string(),
        extra_example: text(v, "extra_example")?.to_string(),
    })
}

fn classes(v: &Value) -> Result<SimClasses, String> {
    let roles: Vec<ClassRoles> = list(v, "roles")?.iter().map(class_roles).collect::<Result<_, _>>()?;
    let generated: Vec<Generated> = list(v, "generated")?
        .iter()
        .map(|g| Ok(Generated { id: leak(text(g, "id")?), enemies: roster(g, "enemies")?, wave: roster(g, "wave")? }))
        .collect::<Result<_, String>>()?;
    let scenarios: Vec<Scenario> = list(v, "scenarios")?.iter().map(scenario).collect::<Result<_, _>>()?;
    Ok(SimClasses {
        ids: static_list(v, "ids")?,
        default_comp: static_list(v, "default_comp")?,
        trio_lead: leak(text(v, "trio_lead")?),
        elite: leak(text(v, "elite")?),
        boss: leak(text(v, "boss")?),
        support_baseline: leak(text(v, "support_baseline")?),
        roles: Box::leak(roles.into_boxed_slice()),
        generated: Box::leak(generated.into_boxed_slice()),
        scenarios: Box::leak(scenarios.into_boxed_slice()),
        s2_enemies: roster(v, "s2_enemies")?,
    })
}

fn class_roles(v: &Value) -> Result<ClassRoles, String> {
    let loadouts: Vec<(&'static str, &'static [&'static str])> = list(v, "loadouts")?.iter().map(|l| Ok((leak(text(l, "role")?), static_list(l, "skills")?))).collect::<Result<_, String>>()?;
    let swaps: Vec<(&'static str, &'static str, &'static str)> =
        list(v, "l7_swaps")?.iter().map(|s| Ok((leak(text(s, "role")?), leak(text(s, "skill")?), leak(text(s, "with")?)))).collect::<Result<_, String>>()?;
    Ok(ClassRoles {
        class: leak(text(v, "class")?),
        primary: leak(text(v, "primary")?),
        secondary: static_list(v, "secondary")?,
        loadouts: Box::leak(loadouts.into_boxed_slice()),
        l7_swaps: Box::leak(swaps.into_boxed_slice()),
    })
}

fn scenario(v: &Value) -> Result<Scenario, String> {
    let block = match list(v, "block")?.as_slice() {
        [] => (0, 0),
        [x, y] => (int(x)?, int(y)?),
        other => return Err(format!("sim config: scenario block {other:?} is not [x, y]")),
    };
    let tier_until = match v.get("tier_until") {
        None | Some(Value::Null) => None,
        Some(t) => Some((int(t.get("level").unwrap_or(&Value::Null))?, int(t.get("slot").unwrap_or(&Value::Null))? as usize, leak(text(t, "tier")?))),
    };
    Ok(Scenario {
        id: leak(text(v, "id")?),
        role: leak(text(v, "role")?),
        allies: static_list(v, "allies")?,
        enemies: roster(v, "enemies")?,
        priority: v.get("priority").and_then(Value::as_bool).unwrap_or(false),
        block,
        tier_until,
    })
}

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// Text key `key` of `v` ("" when missing; Err when it is not text).
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(""),
        Some(Value::String(s)) => Ok(s),
        Some(other) => Err(format!("sim config: `{key}` is not text: {other}")),
    }
}

/// List key `key` of `v` (empty when missing; Err when it is not a list).
fn list<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    static EMPTY: Vec<Value> = Vec::new();
    match v.get(key) {
        None | Some(Value::Null) => Ok(&EMPTY),
        Some(Value::Array(a)) => Ok(a),
        Some(other) => Err(format!("sim config: `{key}` is not a list: {other}")),
    }
}

fn strings(v: &Value, key: &str) -> Result<Vec<&'static str>, String> {
    list(v, key)?.iter().map(|s| s.as_str().map(leak).ok_or_else(|| format!("sim config: `{key}` holds {s}, not text"))).collect()
}

fn owned_list(v: &Value, key: &str) -> Result<Vec<String>, String> {
    list(v, key)?.iter().map(|s| s.as_str().map(str::to_string).ok_or_else(|| format!("sim config: `{key}` holds {s}, not text"))).collect()
}

fn static_list(v: &Value, key: &str) -> Result<&'static [&'static str], String> {
    Ok(Box::leak(strings(v, key)?.into_boxed_slice()))
}

/// Roster key `key` of `v`: `[[class, tier], ...]`.
fn roster(v: &Value, key: &str) -> Result<Roster, String> {
    let units: Vec<(&'static str, &'static str)> = list(v, key)?
        .iter()
        .map(|u| match u.as_array().map(Vec::as_slice) {
            Some([Value::String(c), Value::String(t)]) => Ok((leak(c), leak(t))),
            _ => Err(format!("sim config: `{key}` unit {u} is not [class, tier]")),
        })
        .collect::<Result<_, _>>()?;
    Ok(Box::leak(units.into_boxed_slice()))
}

fn int(v: &Value) -> Result<i32, String> {
    v.as_i64().map(|n| n as i32).ok_or_else(|| format!("sim config: {v} is not a whole number"))
}
