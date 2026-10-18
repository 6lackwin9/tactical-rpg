//! SaveGame and SaveMigrations (scripts/game_state/): one versioned JSON file per slot, the same files Godot
//! writes ({format: "fc_save", version, saved_at, game_state}), slot 0 = autosave, 1..5 manual. Writes go to a temp
//! file then rename; reads migrate then validate and never return a half-loaded state.
//! Every migration is ported (1 -> 10); 2 -> 3 seeds rng_seed from Godot's sorted JSON text, rebuilt by godot_text.

use crate::data::Data;
use crate::json::{self, int, Dict};
use crate::progression;
use crate::state::{default_autoplay, GameState, SUPPLIES_START};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const VERSION: i64 = 10;
pub const FORMAT: &str = "fc_save";
pub const AUTOSAVE_SLOT: i64 = 0;
pub const LAST_SLOT: i64 = 5;
const KEYS: [&str; 14] =
    ["world_id", "location_id", "party", "flags", "quests", "loot_claimed", "position", "explore", "rng_seed", "rolls", "shops", "time", "autoplay", "purses"];

/// The Godot user save folder on this machine (user://saves of the project).
pub fn default_dir() -> PathBuf {
    let base = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    base.join("Godot").join("app_userdata").join("Tactical RPG").join("saves")
}

pub fn path_for(dir: &Path, slot: i64) -> PathBuf {
    dir.join(format!("slot_{slot}.json"))
}

pub fn exists(dir: &Path, slot: i64) -> bool {
    path_for(dir, slot).is_file()
}

pub fn delete(dir: &Path, slot: i64) -> bool {
    exists(dir, slot) && std::fs::remove_file(path_for(dir, slot)).is_ok()
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Problem strings; empty = written. `saved_at` > 0 keeps a given save time.
pub fn write(d: &Data, dir: &Path, state: &GameState, slot: i64, saved_at: i64) -> Vec<String> {
    if !(AUTOSAVE_SLOT..=LAST_SLOT).contains(&slot) {
        return vec!["Save slot must be 0-5.".into()];
    }
    let errors = state.validate(d);
    if !errors.is_empty() {
        return errors;
    }
    let envelope = json::obj([
        ("format", FORMAT.into()),
        ("version", int(VERSION)),
        ("saved_at", int(if saved_at > 0 { saved_at } else { now() })),
        ("game_state", state.to_value()),
    ]);
    if std::fs::create_dir_all(dir).is_err() && !dir.is_dir() {
        return vec![format!("Can't create the save folder '{}'.", dir.display())];
    }
    let final_path = path_for(dir, slot);
    let mut tmp = final_path.with_extension("json.fc_tmp");
    let mut n = 1;
    while tmp.exists() {
        tmp = final_path.with_extension(format!("json.fc_tmp{n}"));
        n += 1;
    }
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    if serde::Serialize::serialize(&json::sorted(&Value::Object(envelope)), &mut ser).is_err() || std::fs::write(&tmp, &buf).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return vec![format!("Can't write '{}'.", final_path.display())];
    }
    if std::fs::rename(&tmp, &final_path).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return vec!["Can't move the save into place (the previous save was kept).".into()];
    }
    Vec::new()
}

pub struct Loaded {
    pub state: GameState,
    pub warnings: Vec<String>,
    pub saved_at: i64,
}

pub fn read(d: &Data, dir: &Path, slot: i64) -> Result<Loaded, Vec<String>> {
    if !exists(dir, slot) {
        return Err(vec![format!("No save in slot {slot}.")]);
    }
    decode(d, &std::fs::read_to_string(path_for(dir, slot)).unwrap_or_default())
}

/// A save file's text: parse, format check, migrate, validate.
pub fn decode(d: &Data, text: &str) -> Result<Loaded, Vec<String>> {
    let Ok(Value::Object(envelope)) = serde_json::from_str::<Value>(text) else {
        return Err(vec!["Save file is corrupt or truncated.".into()]);
    };
    if json::s(&envelope, "format") != FORMAT {
        return Err(vec![format!("Not a save file (format '{}').", json::s(&envelope, "format"))]);
    }
    let (data, warnings) = migrate(d, &envelope)?;
    let state = GameState::from_dict(d, json::dget(&data, "game_state"));
    let errors = state.validate(d);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Loaded { state, warnings, saved_at: json::i(&envelope, "saved_at", 0) })
}

fn quest_entry_ok(key: &str, v: &Value) -> bool {
    let mut one = Dict::new();
    one.insert(key.into(), v.clone());
    crate::state::quest_blob_errors(&one).is_empty()
}

/// JSON.stringify(v, "", true) as Godot writes a value it parsed from a save: keys sorted, no spaces, every number a
/// float ("3.0"; Godot's JSON parser makes all numbers floats), except the quest blob after the 1 -> 2 step, which
/// JsonUtil.normalize turned back into ints (`ints`).
fn godot_text(v: &Value, ints: bool) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f == f.floor() && f.abs() < 1e15 {
                if ints && (n.is_i64() || n.is_u64()) {
                    format!("{}", f as i64)
                } else {
                    format!("{}.0", f as i64)
                }
            } else {
                format!("{f}")
            }
        }
        Value::String(s) => serde_json::to_string(s).unwrap_or_default(),
        Value::Array(a) => format!("[{}]", a.iter().map(|x| godot_text(x, ints)).collect::<Vec<_>>().join(",")),
        Value::Object(o) => {
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort();
            format!("{{{}}}", keys.iter().map(|k| format!("{}:{}", serde_json::to_string(k).unwrap_or_default(), godot_text(&o[*k], ints))).collect::<Vec<_>>().join(","))
        }
    }
}

fn state_text(state: &Dict, quests_int: bool) -> String {
    let mut keys: Vec<&String> = state.keys().collect();
    keys.sort();
    let parts: Vec<String> = keys.iter().map(|k| format!("{}:{}", serde_json::to_string(k).unwrap_or_default(), godot_text(&state[*k], quests_int && k.as_str() == "quests"))).collect();
    format!("{{{}}}", parts.join(","))
}

fn step(d: &Data, version: i64, state: &Dict, quests_int: bool) -> Option<Dict> {
    let mut out = state.clone();
    let party = |out: &mut Dict, f: &dyn Fn(&mut Dict)| {
        if let Some(Value::Object(p)) = out.get_mut("party") {
            f(p);
        }
    };
    let members = |out: &mut Dict, f: &dyn Fn(&mut Dict)| {
        if let Some(Value::Object(p)) = out.get_mut("party") {
            if let Some(Value::Array(ms)) = p.get_mut("members") {
                for m in ms.iter_mut() {
                    if let Value::Object(m) = m {
                        f(m);
                    }
                }
            }
        }
    };
    match version {
        1 => {
            let quests = json::normalize(state.get("quests").unwrap_or(&Value::Null));
            let kept: Dict = json::dict(&quests).iter().filter(|(k, v)| quest_entry_ok(k, v)).map(|(k, v)| (k.clone(), v.clone())).collect();
            out.insert("quests".into(), Value::Object(kept));
            out.insert("loot_claimed".into(), Value::Array(Vec::new()));
        }
        2 => {
            let seed = (aegis_rules::ghash::string_hash(&state_text(state, quests_int)) & 0x7FFF_FFFF) as i64;
            out.insert("rng_seed".into(), int(seed));
            out.insert("position".into(), Value::Array(vec![int(-1), int(-1)]));
            out.insert("explore".into(), Value::Object(crate::state::default_explore()));
            out.insert("rolls".into(), Value::Object(Dict::new()));
            let (world, loc) = (json::s(state, "world_id"), json::s(state, "location_id"));
            let hit = if world.is_empty() || loc.is_empty() { json::empty() } else { d.resolve(&world, "locations", &loc) };
            out.insert("location_id".into(), json::s(hit, "_qid").into());
        }
        3 => {
            party(&mut out, &|p| {
                p.insert("gold".into(), int(0));
            });
            out.insert("shops".into(), Value::Object(Dict::new()));
        }
        4 => {
            out.insert("time".into(), int(0));
        }
        5 => party(&mut out, &|p| {
            p.insert("supplies".into(), int(SUPPLIES_START));
        }),
        6 => {
            out.insert("autoplay".into(), Value::Object(default_autoplay()));
        }
        7 => members(&mut out, &|m| {
            m.insert("pouch".into(), Value::Array(Vec::new()));
        }),
        8 => {
            members(&mut out, &|m| {
                m.insert("hp_lost".into(), int(0));
            });
            if let Some(Value::Object(e)) = out.get_mut("explore") {
                e.insert("last_rest".into(), "".into());
            }
        }
        9 => {
            out.insert("purses".into(), Value::Object(Dict::new()));
        }
        _ => return None,
    }
    Some(out)
}

/// Brings an envelope to VERSION: (envelope, warnings) or the refusal.
pub fn migrate(d: &Data, envelope: &Dict) -> Result<(Dict, Vec<String>), Vec<String>> {
    let mut warnings = Vec::new();
    let mut data = envelope.clone();
    let raw_version = json::get(&data, "version");
    if !raw_version.is_number() {
        return Err(vec!["save has no version".into()]);
    }
    let mut version = json::to_int(raw_version);
    if version > VERSION {
        return Err(vec!["save is from a newer version".into()]);
    }
    if version < 1 {
        return Err(vec![format!("save version {version} is invalid")]);
    }
    let mut state = json::dget(&data, "game_state").clone();
    let mut quests_int = false;
    while version < VERSION {
        if !state.get("quests").map_or(true, Value::is_object) {
            warnings.push("dropped invalid quests blob (not an object)".into());
        }
        let before: Vec<String> = json::dget(&state, "quests").keys().cloned().collect();
        let loc_before = json::s(&state, "location_id");
        let Some(next) = step(d, version, &state, quests_int) else {
            return Err(vec![format!("no migration from save version {version}")]);
        };
        state = next;
        quests_int = quests_int || version == 1;
        let after = json::dget(&state, "quests");
        for k in before {
            if !after.contains_key(&k) {
                warnings.push(format!("dropped invalid quest entry '{k}'"));
            }
        }
        if !loc_before.is_empty() && json::s(&state, "location_id").is_empty() {
            warnings.push(format!("cleared unresolved location_id '{loc_before}'"));
        }
        version += 1;
    }
    let keys: Vec<String> = state.keys().cloned().collect();
    for k in keys {
        if !KEYS.contains(&k.as_str()) {
            warnings.push(format!("dropped unknown game_state key '{k}'"));
            state.shift_remove(&k);
        }
    }
    data.insert("game_state".into(), Value::Object(state));
    data.insert("version".into(), int(version));
    Ok((data, warnings))
}

/// A slot header without loading: {exists, ok, slot, saved_at, version, world_id, location_id, party: [{id, base_unit,
/// level}], error ("unreadable" | "newer version" | "")}.
pub fn peek(d: &Data, dir: &Path, slot: i64) -> Dict {
    let mut out = json::obj([
        ("exists", Value::Bool(exists(dir, slot))),
        ("ok", Value::Bool(false)),
        ("slot", int(slot)),
        ("saved_at", int(0)),
        ("version", int(0)),
        ("world_id", "".into()),
        ("location_id", "".into()),
        ("party", Value::Array(Vec::new())),
        ("error", "".into()),
    ]);
    if !json::b(&out, "exists", false) {
        return out;
    }
    let text = std::fs::read_to_string(path_for(dir, slot)).unwrap_or_default();
    let envelope = match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(e)) if json::s(&e, "format") == FORMAT => e,
        _ => {
            out.insert("error".into(), "unreadable".into());
            return out;
        }
    };
    let version = json::get(&envelope, "version");
    if !version.is_number() || json::to_int(version) < 1 {
        out.insert("error".into(), "unreadable".into());
        return out;
    }
    out.insert("saved_at".into(), int(json::i(&envelope, "saved_at", 0)));
    out.insert("version".into(), int(json::to_int(version)));
    let gs = json::dget(&envelope, "game_state");
    out.insert("world_id".into(), json::s(gs, "world_id").into());
    out.insert("location_id".into(), json::s(gs, "location_id").into());
    let mut party = Vec::new();
    for m in json::aget(json::dget(gs, "party"), "members") {
        let m = json::dict(m);
        let level = if m.contains_key("total_xp") { progression::level_for(d, json::i(m, "total_xp", 0)) } else { 0 };
        party.push(Value::Object(json::obj([("id", json::s(m, "id").into()), ("base_unit", json::s(m, "base_unit").into()), ("level", int(level))])));
    }
    out.insert("party".into(), Value::Array(party));
    if json::to_int(version) > VERSION {
        out.insert("error".into(), "newer version".into());
        return out;
    }
    out.insert("ok".into(), Value::Bool(true));
    out
}

/// The readable slot with the newest saved_at; -1 when none.
pub fn newest_slot(d: &Data, dir: &Path) -> i64 {
    let (mut best, mut best_at) = (-1, -1);
    for slot in AUTOSAVE_SLOT..=LAST_SLOT {
        let head = peek(d, dir, slot);
        if json::b(&head, "ok", false) && json::i(&head, "saved_at", 0) > best_at {
            best = slot;
            best_at = json::i(&head, "saved_at", 0);
        }
    }
    best
}
