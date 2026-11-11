//! SaveGame and SaveMigrations (scripts/game_state/), the game's side: the engine save framework (`aegis_save`) keeps
//! the slot files, the envelope ({format: "fc_save", version, saved_at, game_state}), the atomic write and the
//! migration runner; this module supplies the format, the GameState shape and the release upgrade table (`STEPS`,
//! empty while the game keeps one save version; CLN-47 dropped the development-era steps v1 -> v13). Slot 0 =
//! autosave, 1..5 manual; reads migrate then validate and never return a half-loaded state; a save older than
//! `OLDEST` is refused.

use crate::data::Data;
use crate::progression;
use crate::state::GameState;
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
    read_keyed_with(d, dir, key, &[])
}

/// LAYER-DOWN-1: `read_keyed` with the game's renamed deck keys (`KeyAlias`).
pub fn read_keyed_with(d: &Data, dir: &Path, key: &str, deck_aliases: &[KeyAlias]) -> Result<Loaded, Vec<String>> {
    decode_with(d, &aegis_save::read_keyed(dir, key)?, deck_aliases)
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
    read_with(d, dir, slot, &[])
}

/// LAYER-DOWN-1: a renamed key of a saved `deck`: (the key an older save holds, the `Deck::KEYS` key it now is). The
/// kit names none of a game's old words; a game passes its own list to `read_with`, `read_keyed_with` and
/// `decode_with` (its world façade's `save::read`) so its older saves keep loading.
pub type KeyAlias = (&'static str, &'static str);

/// LAYER-DOWN-1: `read` with the game's renamed deck keys (`KeyAlias`).
pub fn read_with(d: &Data, dir: &Path, slot: i64, deck_aliases: &[KeyAlias]) -> Result<Loaded, Vec<String>> {
    decode_with(d, &aegis_save::read(dir, slot)?, deck_aliases)
}

/// LAYER-DOWN-1: every `deck` object inside `v` (party and departed members): an old key of `aliases` becomes its new
/// key, unless the deck holds the new key too (then both stay and the deck check refuses the old one).
fn rename_deck_keys(v: &mut Value, aliases: &[KeyAlias]) {
    match v {
        Value::Object(o) => {
            for (k, x) in o.iter_mut() {
                if let (true, Value::Object(deck)) = (k == "deck", &mut *x) {
                    for (old, new) in aliases {
                        if !deck.contains_key(*new) {
                            if let Some(list) = deck.shift_remove(*old) {
                                deck.insert((*new).to_string(), list);
                            }
                        }
                    }
                } else {
                    rename_deck_keys(x, aliases);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|x| rename_deck_keys(x, aliases)),
        _ => {}
    }
}

/// A save file's text: parse, format check, migrate, validate (STORYUI1 Q8: a `story.running` the data no longer has
/// is dropped with a warning first; ROSTER1 11.4: bunk use above the cap is a warning, never a refusal; CAMPAIGN1-TM2:
/// a saved travel mode the world no longer has loads in the default mode with a warning; R4-2 M1: a member's saved
/// deck entry the data no longer allows is dropped with a warning, `UnitState::sanitize_deck`; PR4-2: so is a loadout
/// family, rank or worn item, `UnitState::sanitize_loadout`; PK2-S1: so is a unit track, `UnitState::sanitize_tracks`;
/// PK2-S2: so is a body slot entry, `UnitState::sanitize_body`).
pub fn decode(d: &Data, text: &str) -> Result<Loaded, Vec<String>> {
    decode_with(d, text, &[])
}

/// LAYER-DOWN-1: `decode` with the game's renamed deck keys (`KeyAlias`), applied after the migration and before the
/// state is read.
pub fn decode_with(d: &Data, text: &str, deck_aliases: &[KeyAlias]) -> Result<Loaded, Vec<String>> {
    let envelope = aegis_save::open(&SAVES, text)?;
    let (mut data, mut warnings) = migrate(d, &envelope)?;
    if let (false, Some(gs)) = (deck_aliases.is_empty(), data.get_mut("game_state")) {
        rename_deck_keys(gs, deck_aliases);
    }
    let mut state = GameState::from_dict(d, json::dget(&data, "game_state"));
    warnings.extend(crate::story::check_loaded(d, &mut state));
    if let Some(c) = state.campaign.as_mut() {
        warnings.append(&mut c.load_notes);
    }
    // R4-2 M1 (FB-138): a saved deck entry the current data disallows (a skill became a class basic, a family left
    // the path) is dropped with a load warning instead of refusing the save
    // PR4-2 (AUDIT-08): a saved loadout family, rank or worn item the current data disallows is dropped (worn gear
    // to the stash) with a load warning too
    let party = &mut state.party;
    for m in &mut party.members {
        // PK2-S1: a unit track the data no longer has is dropped, a stage past its row's top held there (before the
        // loadout, whose two-handed refusal reads the stage passives)
        warnings.extend(m.sanitize_tracks(d));
        // PK2-S2: a body entry the data no longer allows is dropped (before the deck, which may name a graft's skill)
        warnings.extend(m.sanitize_body(d));
        warnings.extend(m.sanitize_deck(d));
        warnings.extend(m.sanitize_loadout(d, &mut party.stash));
    }
    let errors = state.validate(d);
    if !errors.is_empty() {
        return Err(errors);
    }
    warnings.extend(crate::roster::load_warnings(&state));
    Ok(Loaded { state, warnings, saved_at: json::i(&envelope, "saved_at", 0) })
}

/// One release upgrade step: turns a game_state of its table row's `from` version into `from + 1` (None: refuse the
/// save) and may add load warnings. `C` is what the step reads besides the state (the game's `Data` in `STEPS`).
pub type Step<C> = fn(&C, &Dict, &mut Vec<String>) -> Option<Dict>;

/// The oldest save version this build reads (CLN-47, decision "Save upgrades at game version updates"). The
/// development-era steps v1 -> v13 are gone: a save below this is refused (`older_build_message`). At a released game
/// version update, bump `VERSION`, add the previous release's step to `STEPS` and leave this at that release's version.
pub const OLDEST: i64 = VERSION;

/// The release upgrade table: (from version, step), one row per version from `OLDEST` up to `VERSION - 1`. Empty while
/// the game keeps one save version.
const STEPS: &[(i64, Step<Data>)] = &[];

/// The refusal for a save older than `OLDEST`.
pub fn older_build_message(version: i64) -> String {
    format!("This save is from an older development build (v{version}); start a new game.")
}

/// Brings an envelope to VERSION: (envelope, warnings) or the refusal (older than `OLDEST`, newer than `VERSION`, no
/// version, or a version `STEPS` has no step for).
pub fn migrate(d: &Data, envelope: &Dict) -> Result<(Dict, Vec<String>), Vec<String>> {
    migrate_with(&SAVES, OLDEST, STEPS, d, envelope)
}

/// `migrate` over any format, oldest version and step table (the hook `migrate` runs; unit-tested with a test step).
fn migrate_with<C>(fmt: &Format, oldest: i64, steps: &[(i64, Step<C>)], cx: &C, envelope: &Dict) -> Result<(Dict, Vec<String>), Vec<String>> {
    let raw = json::get(envelope, "version");
    let version = json::to_int(raw);
    if raw.is_number() && (1..oldest).contains(&version) {
        return Err(vec![older_build_message(version)]);
    }
    aegis_save::migrate(fmt, envelope, |from, state, warnings| steps.iter().find(|(v, _)| *v == from).and_then(|(_, step)| step(cx, state, warnings)))
}

/// A slot header without loading: {exists, ok, slot, saved_at, version, world_id, location_id, party: [{id, base_unit,
/// level}], error ("unreadable" | "newer version" |
/// "older version" | "")}.
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
    if version < OLDEST {
        out.insert("error".into(), "older version".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A test release: version 3 reads version 2 saves through one step and refuses version 1.
    const TEST_KEYS: [&str; 2] = ["gold", "badge"];
    const TEST: Format = Format { tag: "test_save", version: 3, first_slot: 0, last_slot: 1, keys: &TEST_KEYS };

    fn add_badge(badge: &String, state: &Dict, warnings: &mut Vec<String>) -> Option<Dict> {
        let mut out = state.clone();
        out.insert("badge".into(), badge.as_str().into());
        warnings.push("added a badge".into());
        Some(out)
    }

    fn envelope(version: i64) -> Dict {
        json::dict(&json!({"format": "test_save", "version": version, "saved_at": 1, "game_state": {"gold": 5}})).clone()
    }

    #[test]
    fn the_hook_runs_a_registered_release_step() {
        let steps: &[(i64, Step<String>)] = &[(2, add_badge)];
        let badge = "gilt".to_string();
        let (out, warnings) = migrate_with(&TEST, 2, steps, &badge, &envelope(2)).expect("a v2 save upgrades");
        assert_eq!(out["version"], json!(3));
        assert_eq!(out["game_state"], json!({"gold": 5, "badge": "gilt"}));
        assert_eq!(warnings, vec!["added a badge".to_string()]);
        let (same, none) = migrate_with(&TEST, 2, steps, &badge, &envelope(3)).expect("the current version reads as is");
        assert_eq!(same["game_state"], json!({"gold": 5}));
        assert!(none.is_empty());
    }

    #[test]
    fn the_hook_refuses_below_oldest_and_a_missing_step() {
        let steps: &[(i64, Step<String>)] = &[(2, add_badge)];
        let badge = String::new();
        assert_eq!(migrate_with(&TEST, 2, steps, &badge, &envelope(1)).err(), Some(vec![older_build_message(1)]));
        assert_eq!(migrate_with(&TEST, 1, steps, &badge, &envelope(1)).err(), Some(vec!["no migration from save version 1".to_string()]));
        assert!(migrate_with(&TEST, 2, steps, &badge, &envelope(4)).is_err(), "a newer save is refused");
    }

    #[test]
    fn the_shipped_table_covers_oldest_to_version() {
        assert!(OLDEST <= VERSION);
        let froms: Vec<i64> = STEPS.iter().map(|(v, _)| *v).collect();
        assert_eq!(froms, (OLDEST..VERSION).collect::<Vec<_>>(), "one step per version from OLDEST to VERSION - 1");
    }
}
