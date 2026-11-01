//! SaveGame and SaveMigrations (scripts/game_state/), the game's side: the engine save framework (`aegis_save`) keeps
//! the slot files, the envelope ({format: "fc_save", version, saved_at, game_state}), the atomic write and the
//! migration runner; this module supplies the format, the GameState shape and every migration step (1 -> 13). Slot
//! 0 = autosave, 1..5 manual; reads migrate then validate and never return a half-loaded state. 2 -> 3 seeds
//! rng_seed from Godot's sorted JSON text, rebuilt by godot_text.

use crate::data::Data;
use crate::progression;
use crate::state::{default_autoplay, GameState, SUPPLIES_START};
use aegis_core::json::{self, int, Dict};
use aegis_save::{Format, Peek};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub use aegis_save::{delete, exists, is_locked, path_for};

pub const VERSION: i64 = 13;
pub const FORMAT: &str = "fc_save";
pub const AUTOSAVE_SLOT: i64 = 0;
pub const LAST_SLOT: i64 = 5;
const KEYS: [&str; 17] = [
    "world_id", "location_id", "party", "flags", "quests", "loot_claimed", "position", "explore", "rng_seed", "rolls", "shops", "time", "autoplay", "purses",
    "reward_paid", "campaign", "story",
];
const SAVES: Format = Format { tag: FORMAT, version: VERSION, first_slot: AUTOSAVE_SLOT, last_slot: LAST_SLOT, keys: &KEYS };

/// The Godot user save folder on this machine (user://saves of the project).
pub fn default_dir() -> PathBuf {
    let base = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    base.join("Godot").join("app_userdata").join("Tactical RPG").join("saves")
}

/// Problem strings; empty = written. `saved_at` > 0 keeps a given save time. Refused while the run is over
/// (CAMPAIGN1 F3), so no slot ever holds a run-over state.
pub fn write(d: &Data, dir: &Path, state: &GameState, slot: i64, saved_at: i64) -> Vec<String> {
    let over = crate::campaign::save_refusal(state);
    if !over.is_empty() {
        return vec![over.to_string()];
    }
    aegis_save::write(&SAVES, dir, slot, saved_at, || {
        let errors = state.validate(d);
        if errors.is_empty() {
            Ok(state.to_value())
        } else {
            Err(errors)
        }
    })
}

/// CAMPAIGN1 F5: writes the keyed save `key` (an act-start save, `campaign::act_save_key`; `aegis_save::write_keyed`:
/// never a slot, a later write with the key replaces it). Problem strings; empty = written. Refused while the run is
/// over (F3), like `write`.
pub fn write_keyed(d: &Data, dir: &Path, state: &GameState, key: &str, saved_at: i64) -> Vec<String> {
    let over = crate::campaign::save_refusal(state);
    if !over.is_empty() {
        return vec![over.to_string()];
    }
    aegis_save::write_keyed(&SAVES, dir, key, saved_at, || {
        let errors = state.validate(d);
        if errors.is_empty() {
            Ok(state.to_value())
        } else {
            Err(errors)
        }
    })
}

/// CAMPAIGN1 F4: reads the keyed save `key` like `read` (migrate, then validate).
pub fn read_keyed(d: &Data, dir: &Path, key: &str) -> Result<Loaded, Vec<String>> {
    decode(d, &aegis_save::read_keyed(dir, key)?)
}

/// Whether the keyed save `key` exists (a bad key has none).
pub fn exists_keyed(dir: &Path, key: &str) -> bool {
    aegis_save::exists_keyed(dir, key)
}

/// Locks or unlocks a manual slot's save (`aegis_save::set_locked`: the autosave is never locked; a locked slot
/// refuses `write` and `delete`).
pub fn set_locked(dir: &Path, slot: i64, on: bool) -> Result<(), String> {
    aegis_save::set_locked(&SAVES, dir, slot, on)
}

pub struct Loaded {
    pub state: GameState,
    pub warnings: Vec<String>,
    pub saved_at: i64,
}

pub fn read(d: &Data, dir: &Path, slot: i64) -> Result<Loaded, Vec<String>> {
    decode(d, &aegis_save::read(dir, slot)?)
}

/// A save file's text: parse, format check, migrate, validate (STORYUI1 Q8: a `story.running` the data no longer has
/// is dropped with a warning first; ROSTER1 11.4: bunk use above the cap is a warning, never a refusal; CAMPAIGN1-TM2:
/// a saved travel mode the world no longer has loads in the default mode with a warning).
pub fn decode(d: &Data, text: &str) -> Result<Loaded, Vec<String>> {
    let envelope = aegis_save::open(&SAVES, text)?;
    let (data, mut warnings) = migrate(d, &envelope)?;
    let mut state = GameState::from_dict(d, json::dget(&data, "game_state"));
    warnings.extend(crate::story::check_loaded(d, &mut state));
    if let Some(c) = state.campaign.as_mut() {
        warnings.append(&mut c.load_notes);
    }
    let errors = state.validate(d);
    if !errors.is_empty() {
        return Err(errors);
    }
    warnings.extend(crate::roster::load_warnings(&state));
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
            let seed = (aegis_core::ghash::string_hash(&state_text(state, quests_int)) & 0x7FFF_FFFF) as i64;
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
        // SKILLUI1: members may carry `loadout.ranks`; nothing to add (absent = the highest unlocked rank). The
        // bump makes an older build refuse such a save ("newer version") instead of dropping the ranks.
        10 => {}
        // REWARD1: `reward_paid` = {}, then every roster key of a `once` purse already paid (a story fight won before
        // REWARD1 does not pay again).
        11 => {
            let mut paid = Dict::new();
            for (qid, wins) in json::dget(&out, "purses").clone() {
                let enc = d.resolve("", "encounters", &qid);
                if json::to_int(&wins) < 1 || enc.is_empty() || !crate::purse::read(enc).is_some_and(|p| p.2) {
                    continue;
                }
                let keys: Vec<String> = crate::battle::roster(d, &json::s(enc, "_world"), &qid).into_iter().map(|f| f.key).collect();
                paid.insert(qid, json::strs(&keys));
            }
            out.insert("reward_paid".into(), Value::Object(paid));
        }
        // CAMPAIGN1 11.5: a campaign world may save `campaign`; nothing to add (absent = a fresh campaign in a
        // campaign world, none elsewhere). The bump makes an older build refuse such a save instead of dropping it.
        // STORYUI1 4.8 shares this unreleased bump (ROSTER1 11.4): `story` may be saved; absent reads empty.
        // RULESET1 9 D5 shares it too: a party member may save `deck`; absent reads the empty deck (auto).
        // ROSTER1 11.4 shares it too: a member may save `conditions`, the party `departed`; absent reads none.
        12 => {}
        _ => return None,
    }
    Some(out)
}

/// Brings an envelope to VERSION: (envelope, warnings) or the refusal. Each step reports the quest entries it dropped
/// and a location_id it could not resolve.
pub fn migrate(d: &Data, envelope: &Dict) -> Result<(Dict, Vec<String>), Vec<String>> {
    let mut quests_int = false;
    aegis_save::migrate(&SAVES, envelope, |version, state, warnings| {
        if !state.get("quests").map_or(true, Value::is_object) {
            warnings.push("dropped invalid quests blob (not an object)".into());
        }
        let before: Vec<String> = json::dget(state, "quests").keys().cloned().collect();
        let loc_before = json::s(state, "location_id");
        let next = step(d, version, state, quests_int)?;
        quests_int = quests_int || version == 1;
        let after = json::dget(&next, "quests");
        for k in before {
            if !after.contains_key(&k) {
                warnings.push(format!("dropped invalid quest entry '{k}'"));
            }
        }
        if !loc_before.is_empty() && json::s(&next, "location_id").is_empty() {
            warnings.push(format!("cleared unresolved location_id '{loc_before}'"));
        }
        Some(next)
    })
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
    let envelope = match aegis_save::peek(&SAVES, dir, slot) {
        Peek::Missing => return out,
        Peek::Unreadable => {
            out.insert("error".into(), "unreadable".into());
            return out;
        }
        Peek::Found(e) => e,
    };
    let version = json::to_int(json::get(&envelope, "version"));
    out.insert("saved_at".into(), int(json::i(&envelope, "saved_at", 0)));
    out.insert("version".into(), int(version));
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
    if version > VERSION {
        out.insert("error".into(), "newer version".into());
        return out;
    }
    out.insert("ok".into(), Value::Bool(true));
    out
}

/// The readable slot with the newest saved_at; -1 when none.
pub fn newest_slot(d: &Data, dir: &Path) -> i64 {
    aegis_save::newest_slot(&SAVES, |slot| {
        let head = peek(d, dir, slot);
        json::b(&head, "ok", false).then(|| json::i(&head, "saved_at", 0))
    })
}
