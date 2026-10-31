//! Data independence: the engine's own loader (emberwake_content::source) reads a data folder and must build exactly what
//! Godot's catalogs build from it, as recorded by games/emberwake/parity/export.sh from legacy/ (so the loader reads
//! legacy/data): every catalog kind (skill ranks expanded, items resolved against their families), every world (manifest, lineage, own entries) and the game data
//! (maps, quests, services, XP curve, item families). Keys, key order and int / float types must match; floats match
//! to 1e-9 (Godot's JSON.stringify writes fewer digits than it computes with). The schema copy must match the export.

use emberwake_content::source;
use serde_json::Value;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out")
}

fn data_dir() -> PathBuf {
    // the data Godot read for the exports: the frozen copy (the live data/ is checked by tests/store.rs)
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../legacy/data")
}

/// Retired parity: fields added after the Godot freeze, by (schema section, kind, key). They are taken off the engine's
/// side (catalog rows, world entries, game data rows, schema rows) before comparing; each is named in docs/HISTORY.md. REWARD1
/// (2026-10-21): the unit loot table and a world encounter's key loot and objective reward. ASSETORG2 (2026-10-22): a
/// unit's and a character's palette and portrait tier. PROPS1 (2026-10-23): a map template's props (section "game": the
/// game data beside the catalogs; Godot's MapCatalog had no such field). RULESET1-R (2026-10-25): a world manifest's
/// `ruleset` and `plan_reveal_flag` (kind "world": taken off every world's `manifest`). RULESET1-S (2026-10-25): the
/// timeline fields of a skill (`ap_cost`, `recovery_blocks`, `round_intent`, `intent_buff`, `stagger`, `windup_poise`,
/// `unbreakable`, `sure_break`, `armor_piercing`), a unit (`poise`, `deck`, `default_intent`) and a passive (`slot`).
/// ENCOUNTER2-T (2026-10-25): an encounter's authored terrain `rows` (catalog and world encounters). CAMPAIGN1-S
/// (2026-10-25): a world manifest's `campaign`, `purse`, `rest` and `defeat` (kind "world"). STORYUI1-S (2026-10-26):
/// a world quest's board fields `board`, `track`, `show_locked`, `closed_text` and a manifest's `start_sequence`.
/// HOOKS1-S (2026-10-27): a skill's combat hook fields (`forced_move`, `hazard`, `triggers`, `riders`, `meters`,
/// `hp_cost`, `ignore_guard`), a consumable's `hazard`, a unit's `tactics`, an encounter's `arena_hazards` (catalog and
/// world encounters) and a world manifest's `classes`. ENCOUNTER2-H (2026-10-27): an encounter's `unkillable`,
/// `timeout_outcome` and `stay_behind` (catalog and world encounters). ROSTER1-L (2026-10-31): a world manifest's
/// `roster`, a world character's `keepsake` and a world encounter's `party_min`. INPUT1-M (2026-11-01): a skill's
/// `position` (the ruleset fields ride the `rulesets` kind, already after Godot). CAMPAIGN1-TM (2026-11-01): a world
/// manifest's `travel_modes`.
const AFTER_GODOT: [(&str, &str, &str); 57] = [
    ("content", "units", "loot"),
    ("world", "encounters", "key_loot"),
    ("world", "encounters", "objective_reward"),
    ("content", "units", "palette"),
    ("content", "units", "portrait_tier"),
    ("world", "characters", "palette"),
    ("world", "characters", "portrait_tier"),
    ("game", "maps", "props"),
    ("world", "world", "ruleset"),
    ("world", "world", "plan_reveal_flag"),
    ("content", "skills", "ap_cost"),
    ("content", "skills", "recovery_blocks"),
    ("content", "skills", "round_intent"),
    ("content", "skills", "intent_buff"),
    ("content", "skills", "stagger"),
    ("content", "skills", "windup_poise"),
    ("content", "skills", "unbreakable"),
    ("content", "skills", "sure_break"),
    ("content", "skills", "armor_piercing"),
    ("content", "skills", "position"),
    ("content", "units", "poise"),
    ("content", "units", "deck"),
    ("content", "units", "default_intent"),
    ("content", "passives", "slot"),
    ("content", "encounters", "rows"),
    ("world", "encounters", "rows"),
    ("world", "world", "campaign"),
    ("world", "world", "purse"),
    ("world", "world", "rest"),
    ("world", "world", "defeat"),
    ("world", "quests", "board"),
    ("world", "quests", "track"),
    ("world", "quests", "show_locked"),
    ("world", "quests", "closed_text"),
    ("world", "world", "start_sequence"),
    ("content", "skills", "forced_move"),
    ("content", "skills", "hazard"),
    ("content", "skills", "triggers"),
    ("content", "skills", "riders"),
    ("content", "skills", "meters"),
    ("content", "skills", "hp_cost"),
    ("content", "skills", "ignore_guard"),
    ("content", "items", "hazard"),
    ("content", "units", "tactics"),
    ("content", "encounters", "arena_hazards"),
    ("world", "encounters", "arena_hazards"),
    ("content", "encounters", "unkillable"),
    ("content", "encounters", "timeout_outcome"),
    ("content", "encounters", "stay_behind"),
    ("world", "encounters", "unkillable"),
    ("world", "encounters", "timeout_outcome"),
    ("world", "encounters", "stay_behind"),
    ("world", "world", "classes"),
    ("world", "world", "roster"),
    ("world", "characters", "keepsake"),
    ("world", "encounters", "party_min"),
    ("world", "world", "travel_modes"),
];
/// Retired parity, option level: enum and multi-pick options added after the freeze, by (section, kind, key, option);
/// the option leaves the engine's schema row before comparing (every other option and its order is still compared);
/// each is named in docs/HISTORY.md. RULESET1-S (2026-10-25): the weapon types `polearm` and `scourge` (TQ8) and the
/// family engine rule `reach` (K1). CAST1-S (2026-10-26): the weapon types `censer`, `bone_saw` and `sprayer`. HOOKS1-S
/// (2026-10-27): a unit's target preferences `SOFTEST` and `TOUGHEST`.
const OPTIONS_AFTER_GODOT: [(&str, &str, &str, &str); 8] = [
    ("content", "items", "weapon_type", "polearm"),
    ("content", "items", "weapon_type", "scourge"),
    ("content", "items", "weapon_type", "censer"),
    ("content", "items", "weapon_type", "bone_saw"),
    ("content", "items", "weapon_type", "sprayer"),
    ("content", "item_families", "engine_rules", "reach"),
    ("content", "units", "target_pref", "SOFTEST"),
    ("content", "units", "target_pref", "TOUGHEST"),
];
/// Retired parity, kind level (the twin of AFTER_GODOT): schema kinds added after the freeze, by (section, kind); the
/// whole kind leaves the engine's schema before comparing; each is named in docs/HISTORY.md. DERIVE1 (2026-10-25): the
/// derived-stat rows (Godot had no such kind; its catalogs are compared per CATALOG_KINDS, which do not list it).
/// RULESET1-R (2026-10-25): the ruleset rows (the same). CAMPAIGN1-S (2026-10-25): the world kinds `meters`,
/// `checkpoints` and `triggers` (WorldSchema.KINDS had none; every world's `own` lists leave the loader's worlds too).
/// STORYUI1-S (2026-10-26): the world kinds `vignettes`, `sequences` and `boards` (the same). PORTRAITFX1-S
/// (2026-10-26): the portrait cue rows (as the derived stats). HOOKS1-S (2026-10-27): the combat hook kinds `hazards`,
/// `unit_meters` and `zones` (the same).
const KINDS_AFTER_GODOT: [(&str, &str); 12] = [
    ("content", "derived_stats"),
    ("content", "rulesets"),
    ("content", "portrait_cues"),
    ("content", "hazards"),
    ("content", "unit_meters"),
    ("content", "zones"),
    ("world", "meters"),
    ("world", "checkpoints"),
    ("world", "triggers"),
    ("world", "vignettes"),
    ("world", "sequences"),
    ("world", "boards"),
];
/// Schema rows whose help text or options changed after the freeze (their `help` and `options_from` are not
/// compared). REWARD1: the purse rows. ASSETORG2: the art rows (bodies of the asset layers, not Godot's art keys) and
/// the item icon rows. PHASEFOLD: the phases rows (the arrival trigger). PORTRAITFX1-S: the dialogue nodes row (a node's
/// `expression`). HOOKS1-S: a skill's `exploit` (the execute entry), the item and passive `effects` (the hook effect
/// types) and a unit's `target_pref` (SOFTEST, TOUGHEST); the phases rows (the deck effect, ai tactics) are listed already.
/// HOOKS1-Z: a route's road fight `zone` (a code zone or a `zones` row). ENCOUNTER2-C1: the phases rows again (the `talk`
/// effect, a round trigger's `block`, the `downed` and `broken` triggers), listed already.
const HELP_CHANGED: [(&str, &str, &str); 15] = [
    ("content", "encounters", "purse"),
    ("world", "encounters", "purse"),
    ("content", "classes", "art"),
    ("content", "units", "art"),
    ("world", "characters", "art"),
    ("content", "item_families", "art"),
    ("content", "items", "icon_id"),
    ("content", "encounters", "phases"),
    ("world", "encounters", "phases"),
    ("world", "dialogues", "nodes"),
    ("content", "skills", "exploit"),
    ("content", "items", "effects"),
    ("content", "passives", "effects"),
    ("content", "units", "target_pref"),
    ("world", "routes", "zone"),
];

/// Retired parity (PHASEFOLD, HISTORY 2026-10-22): the single reinforcement wave became an arrival phase. Godot's
/// encounter rows (catalog and worlds) are folded by the engine's `fold_legacy_wave` before comparing (the four
/// `wave_*` keys become the arrival phase at the front of `phases`), and Godot's four wave schema rows per section
/// leave its schema; every other key, row and value is still compared.
const FOLDED_SCHEMA_ROWS: [&str; 4] = emberwake_battle::encounter::LEGACY_WAVE_KEYS;

/// Godot's schema without the folded wave rows of both encounter sections.
fn godot_schema_folded(schema: &mut Value) {
    for section in ["content", "world"] {
        if let Some(Value::Array(rows)) = schema.get_mut(section).and_then(|s| s.get_mut("encounters")) {
            rows.retain(|r| !FOLDED_SCHEMA_ROWS.iter().any(|k| r["key"] == *k));
        }
    }
}

/// `v` without the AFTER_GODOT keys of `section`: on every object in an array under a key named after the kind, and
/// (kind "world") on every world's `manifest` object; and (section "world") without the KINDS_AFTER_GODOT world kinds
/// in every world's `own` object.
fn strip_after_godot(v: &mut Value, section: &str) {
    match v {
        Value::Object(o) => {
            if let Some(Value::Object(m)) = o.get_mut("manifest").filter(|_| section == "world") {
                for (_, _, key) in AFTER_GODOT.iter().filter(|(s, kind, _)| *s == "world" && *kind == "world") {
                    m.shift_remove(*key);
                }
            }
            if let Some(Value::Object(own)) = o.get_mut("own").filter(|_| section == "world") {
                for (_, kind) in KINDS_AFTER_GODOT.iter().filter(|(s, _)| *s == "world") {
                    assert!(own.shift_remove(*kind).is_some(), "the loader's world has no own {kind} list to strip");
                }
            }
            for (k, x) in o.iter_mut() {
                let keys: Vec<&str> = AFTER_GODOT.iter().filter(|(s, kind, _)| *s == section && kind == k).map(|t| t.2).collect();
                if let (false, Value::Array(rows)) = (keys.is_empty(), &mut *x) {
                    for row in rows.iter_mut().filter_map(Value::as_object_mut) {
                        for key in &keys {
                            row.shift_remove(*key);
                        }
                    }
                }
                strip_after_godot(x, section);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|x| strip_after_godot(x, section)),
        _ => {}
    }
}

/// The schema without the KINDS_AFTER_GODOT kinds, the AFTER_GODOT rows, the OPTIONS_AFTER_GODOT options and the
/// HELP_CHANGED help texts.
fn schema_at_freeze(schema: &Value) -> Value {
    let mut out = schema.clone();
    for (section, kind) in KINDS_AFTER_GODOT {
        let removed = out.get_mut(section).and_then(Value::as_object_mut).and_then(|s| s.shift_remove(kind));
        assert!(removed.is_some(), "the engine schema has no {section}.{kind} to strip");
    }
    for (section, kind, key) in AFTER_GODOT {
        if let Some(Value::Array(rows)) = out.get_mut(section).and_then(|s| s.get_mut(kind)) {
            rows.retain(|r| r["key"] != key);
        }
    }
    for (section, kind, key, option) in OPTIONS_AFTER_GODOT {
        let row = out.get_mut(section).and_then(|s| s.get_mut(kind)).and_then(Value::as_array_mut).and_then(|rows| rows.iter_mut().find(|r| r["key"] == key));
        let options = row.and_then(|r| r.get_mut("options")).and_then(Value::as_array_mut).expect("an options row to strip");
        let before = options.len();
        options.retain(|o| o != option);
        assert_eq!(options.len() + 1, before, "the engine schema's {section}.{kind}.{key} has no option {option} to strip");
    }
    unhelp(&mut out);
    out
}

fn unhelp(schema: &mut Value) {
    for (section, kind, key) in HELP_CHANGED {
        if let Some(Value::Array(rows)) = schema.get_mut(section).and_then(|s| s.get_mut(kind)) {
            for r in rows.iter_mut().filter(|r| r["key"] == key).filter_map(Value::as_object_mut) {
                r.shift_remove("help");
                r.shift_remove("options_from");
            }
        }
    }
}

fn read(name: &str) -> Option<Value> {
    let text = std::fs::read_to_string(out_dir().join(name)).ok()?;
    Some(serde_json::from_str(&text).unwrap())
}

/// The first difference between Godot's value `g` and the engine's `r` ("" = equal).
fn diff(at: &str, g: &Value, r: &Value) -> String {
    match (g, r) {
        (Value::Number(a), Value::Number(b)) => {
            if a.is_f64() != b.is_f64() {
                return format!("{at}: number type differs: godot {a}, engine {b}");
            }
            let (x, y) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            if (x - y).abs() > 1e-9 * x.abs().max(1.0) {
                return format!("{at}: godot {a}, engine {b}");
            }
            String::new()
        }
        (Value::Object(a), Value::Object(b)) => {
            let (ka, kb): (Vec<&String>, Vec<&String>) = (a.keys().collect(), b.keys().collect());
            if ka != kb {
                return format!("{at}: keys differ:\n  godot  {ka:?}\n  engine {kb:?}");
            }
            for (k, v) in a {
                let d = diff(&format!("{at}.{k}"), v, &b[k]);
                if !d.is_empty() {
                    return d;
                }
            }
            String::new()
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return format!("{at}: length godot {}, engine {}", a.len(), b.len());
            }
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                let id = x.get("id").and_then(Value::as_str).map(|s| format!("[{i} {s}]")).unwrap_or_else(|| format!("[{i}]"));
                let d = diff(&format!("{at}{id}"), x, y);
                if !d.is_empty() {
                    return d;
                }
            }
            String::new()
        }
        _ if g == r => String::new(),
        _ => format!("{at}: godot {g}, engine {r}"),
    }
}

#[test]
fn engine_loader_matches_godot_catalogs() {
    let Some(mut catalog) = read("catalog.json") else {
        eprintln!("no parity/out/catalog.json: run games/emberwake/parity/export.sh");
        return;
    };
    let mut src = source::load(&data_dir());
    assert!(src.errors.is_empty(), "load errors: {:?}", src.errors);
    strip_after_godot(&mut src.catalog, "content");
    strip_after_godot(&mut src.worlds, "world");
    strip_after_godot(&mut src.game, "game");
    let mut worlds = read("worlds.json").unwrap();
    source::fold_legacy_waves(&mut catalog, &mut worlds);
    let mut problems = Vec::new();
    for kind in source::CATALOG_KINDS {
        problems.push(diff(&format!("catalog.{kind}"), &catalog[kind], &src.catalog[kind]));
    }
    problems.push(diff("worlds", &worlds, &src.worlds));
    let game = read("game_data.json").unwrap();
    for key in ["maps", "quests", "services", "xp_curve", "xp_for_level", "item_families"] {
        problems.push(diff(&format!("game.{key}"), &game[key], &src.game[key]));
    }
    if let Some(mut schema) = read("schema.json") {
        godot_schema_folded(&mut schema);
        unhelp(&mut schema);
        problems.push(diff("schema", &schema, &schema_at_freeze(source::schema())));
    }
    problems.retain(|p| !p.is_empty());
    let rows: usize = source::CATALOG_KINDS.iter().map(|k| src.catalog[*k].as_array().map_or(0, |a| a.len())).sum();
    eprintln!("catalog rows: {rows}, worlds: {}", src.worlds.as_object().map_or(0, |w| w.len()));
    assert!(problems.is_empty(), "{} differences; first:\n{}", problems.len(), problems.iter().take(4).cloned().collect::<Vec<_>>().join("\n"));
}

/// Built from the engine's loader, the game data plays the same: playable worlds as Godot lists them.
#[test]
fn engine_loader_data_is_playable() {
    let Some(game) = read("game_data.json") else { return };
    let src = source::load(&data_dir());
    let d = emberwake_world::Data::from_values(&src.catalog, &src.worlds, &src.game);
    let playable: Vec<String> = d.world_ids().into_iter().filter(|w| emberwake_world::new_game::playable_errors(&d, w).is_empty()).collect();
    let godot: Vec<String> = game["playable_worlds"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    assert_eq!(playable, godot);
}
