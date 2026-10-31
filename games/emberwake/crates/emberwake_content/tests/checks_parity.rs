//! The content checks against Godot's (games/emberwake/parity/godot/checks.gd): the frozen Godot data (legacy/data,
//! the files the recording read) loaded by the engine's loader, then every recorded case (an entry with one field set
//! wrong, or a hand-made one) checked again, errors and warnings compared message for message; and the whole-catalog
//! reports. CHECKS_SHOW=<n> prints n differences per kind.

use emberwake_content::check::{self, View};
use emberwake_content::source;
use serde_json::Value;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().map(|s| s.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default()
}

/// Retired parity (REWARD1, HISTORY 2026-10-21): `purse.loot` moved to the encounter's `key_loot`. The messages about
/// it are left out on both sides (Godot's "Purse loot ..." checks, the engine's refusal of the key), in the recorded
/// cases and in the whole-catalog and world reports; every other message of those cases is still compared.
fn kept(messages: Vec<String>) -> Vec<String> {
    messages.into_iter().filter(|m| !retired(m)).map(|m| at_freeze(&m)).collect()
}

/// Retired parity (RULESET1-S, HISTORY 2026-10-25): list items added after the freeze, named in the messages that list
/// a closed set: the gear types `polearm` and `scourge`, the family engine rule `reach`, the passive effect type
/// `stagger_taken`, the buffs `steadfast` and `stealth`. Each ", <item>" ending at ",", ")", "." or the end of a
/// message is taken out (both sides, so a Godot text that names one stays compared); every other word is compared.
/// CAMPAIGN1-S (HISTORY 2026-10-25): the campaign effect kinds in the dialogue effect list. STORYUI1-S (HISTORY
/// 2026-10-26): the story effect kinds in the same list and the service line `board`. CAST1-S (HISTORY 2026-10-26): the
/// gear types `censer`, `bone_saw` and `sprayer`. HOOKS1-S (HISTORY 2026-10-27): the passive and item effect types
/// `forced_move`, `trigger` and `meter`, the target preferences `SOFTEST` and `TOUGHEST` (a unit's and a phase ai's).
const LIST_ITEMS_AFTER_GODOT: [&str; 27] = [
    "polearm", "scourge", "censer", "bone_saw", "sprayer", "reach", "stagger_taken", "steadfast", "stealth", "meter_add", "meter_set", "meter_show", "clock_start", "clock_set", "clock_stop", "set_act", "rest", "end", "vignette", "sequence", "open_board", "board",
    "forced_move", "trigger", "meter", "SOFTEST", "TOUGHEST",
];

fn at_freeze(message: &str) -> String {
    let mut out = message.to_string();
    for item in LIST_ITEMS_AFTER_GODOT {
        let needle = format!(", {item}");
        let mut from = 0;
        while let Some(at) = out[from..].find(&needle).map(|i| i + from) {
            let end = at + needle.len();
            if out[end..].chars().next().is_none_or(|c| matches!(c, ',' | ')' | '.')) {
                out.replace_range(at..end, "");
                from = at;
            } else {
                from = end;
            }
        }
    }
    out
}

fn retired(message: &str) -> bool {
    message.starts_with("Purse loot '") || message == "Purse key 'loot' moved to key_loot (REWARD1)." || art_retired(message) || round_floor_retired(message)
}

/// Retired parity (ENCOUNTER2-C1, HISTORY 2026-11-01): a phase `round` trigger may name round 1 (a battle talk opens
/// round 1, ENCOUNTER2_C PT1), so the range message reads "round must be 1..N." where Godot's read "2..N.". The
/// "Phase '<p>': round must be ..." message leaves the comparison on both sides (the recorded encounters case
/// "Bad Id"); the hp_below `or_round` floor stays 2 and its message is still compared.
fn round_floor_retired(message: &str) -> bool {
    message.starts_with("Phase ") && message.contains("': round must be ")
}

/// Retired parity (ASSETORG2, HISTORY 2026-10-22): the art checks read the asset layers now, not Godot's asset manifest
/// keys. Godot's "Art '<k>' isn't a known token/portrait key." and "Icon '<k>' isn't a known item icon key." leave the
/// comparison, and the engine's view of the frozen data has no asset layers (its A1-A6 checks are off here).
fn art_retired(message: &str) -> bool {
    (message.starts_with("Art '") && message.ends_with("' isn't a known token/portrait key."))
        || (message.starts_with("Icon '") && message.ends_with("' isn't a known item icon key."))
        || (message.starts_with("Art: '") && message.contains("' is not one of"))
}

/// The frozen data's checks view: no asset layers (ASSETORG2 retired the art keys).
fn frozen_view(src: &source::Source) -> View {
    View::new(&src.catalog, &src.worlds, &src.game, aegis_core::assets::Layers::default())
}

fn without_retired(r: check::Report) -> check::Report {
    r.into_iter()
        .map(|(k, l)| (k, kept(l)))
        .filter(|(_, l)| !l.is_empty())
        .collect()
}

#[test]
fn checks_report_what_godot_reported() {
    let path = root().join("games/emberwake/parity/out/checks.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("no parity/out/checks.json: run games/emberwake/parity/checks.sh");
        return;
    };
    let doc: Value = serde_json::from_str(&text).unwrap();
    let src = source::load(&root().join("legacy/data"));
    assert!(src.errors.is_empty(), "legacy/data loads cleanly: {:?}", src.errors);
    let view = frozen_view(&src);
    let show: usize = std::env::var("CHECKS_SHOW").ok().and_then(|s| s.parse().ok()).unwrap_or(2);
    let kinds: Vec<&str> = check::PORTED.to_vec();
    let mut failures = Vec::new();
    let mut per_kind: indexmap::IndexMap<String, (usize, usize)> = indexmap::IndexMap::new();
    for c in doc["cases"].as_array().unwrap() {
        let kind = c["kind"].as_str().unwrap();
        if c["world"] != "" || !kinds.contains(&kind) {
            continue;
        }
        let entry = c["entry"].as_object().unwrap();
        let current = c["current"].as_str().unwrap();
        let (errors, warnings) = (kept(check::validate_entry(&view, kind, entry, current)), kept(check::warnings_for(&view, kind, entry)));
        let ok = errors == kept(strings(&c["errors"])) && warnings == kept(strings(&c["warnings"]));
        let slot = per_kind.entry(kind.to_string()).or_default();
        slot.0 += 1;
        if !ok {
            slot.1 += 1;
            if slot.1 <= show {
                failures.push(format!("{kind} {current}:\n  godot errors {:?}\n  rust  errors {errors:?}\n  godot warnings {:?}\n  rust  warnings {warnings:?}", strings(&c["errors"]), strings(&c["warnings"])));
            }
        }
    }
    for (k, (n, bad)) in &per_kind {
        eprintln!("{k}: {} of {n} cases match", n - bad);
    }
    assert!(per_kind.values().all(|(_, bad)| *bad == 0), "{}", failures.join("\n"));
}

fn report(v: &Value) -> check::Report {
    v.as_object().map(|o| o.iter().map(|(k, l)| (k.clone(), strings(l))).collect()).unwrap_or_default()
}

/// The whole-catalog reports of the shipped data (validate_all, warnings_all), entry by entry.
#[test]
fn catalog_reports_match_godot() {
    let Ok(text) = std::fs::read_to_string(root().join("games/emberwake/parity/out/checks.json")) else {
        eprintln!("no parity/out/checks.json: run games/emberwake/parity/checks.sh");
        return;
    };
    let doc: Value = serde_json::from_str(&text).unwrap();
    let src = source::load(&root().join("legacy/data"));
    let mut view = frozen_view(&src);
    view.terrain_file = root().join("legacy/data/catalog/terrain.json").exists();
    for (name, ours, theirs) in [("errors", check::validate_all(&view), report(&doc["catalog"]["errors"])), ("warnings", check::warnings_all(&view), report(&doc["catalog"]["warnings"]))] {
        let (ours, theirs) = (without_retired(ours), without_retired(theirs));
        let keys: Vec<&String> = theirs.keys().chain(ours.keys()).collect();
        let diffs: Vec<String> = keys.iter().filter(|k| ours.get(**k) != theirs.get(**k)).map(|k| format!("{k}:\n  godot {:?}\n  rust  {:?}", theirs.get(*k), ours.get(*k))).collect();
        eprintln!("catalog {name}: {} godot keys, {} differ", theirs.len(), diffs.len());
        assert!(diffs.is_empty(), "catalog {name}:\n{}", diffs.join("\n"));
    }
}

/// Every recorded world case (an own world entry with one field set wrong) checked again, and the world reports.
#[test]
fn world_checks_match_godot() {
    let Ok(text) = std::fs::read_to_string(root().join("games/emberwake/parity/out/checks.json")) else {
        eprintln!("no parity/out/checks.json: run games/emberwake/parity/checks.sh");
        return;
    };
    let doc: Value = serde_json::from_str(&text).unwrap();
    let src = source::load(&root().join("legacy/data"));
    let view = frozen_view(&src);
    let show: usize = std::env::var("CHECKS_SHOW").ok().and_then(|s| s.parse().ok()).unwrap_or(2);
    let mut per_kind: indexmap::IndexMap<String, (usize, usize)> = indexmap::IndexMap::new();
    let mut failures = Vec::new();
    for c in doc["cases"].as_array().unwrap().iter().filter(|c| c["world"] != "") {
        let (w, kind) = (c["world"].as_str().unwrap(), c["kind"].as_str().unwrap());
        let (entry, current) = (c["entry"].as_object().unwrap(), c["current"].as_str().unwrap());
        let errors = kept(check::world::validate_entry(&view, w, kind, entry, current));
        let blocking = kept(check::world::blocking_errors(&view, w, kind, entry, current));
        let slot = per_kind.entry(kind.to_string()).or_default();
        slot.0 += 1;
        if errors != kept(strings(&c["errors"])) || blocking != kept(strings(&c["blocking"])) {
            slot.1 += 1;
            if slot.1 <= show {
                failures.push(format!("{w} {kind} {}:\n  godot {:?}\n  rust  {errors:?}\n  godot blocking {:?}\n  rust  blocking {blocking:?}", c["current"], strings(&c["errors"]), strings(&c["blocking"])));
            }
        }
    }
    for (k, (n, bad)) in &per_kind {
        eprintln!("world {k}: {} of {n} cases match", n - bad);
    }
    assert!(per_kind.values().all(|(_, bad)| *bad == 0), "{}", failures.join("\n"));
    for (name, ours, theirs) in [("errors", check::world::validate_all(&view), report(&doc["worlds"]["errors"])), ("warnings", check::world::warnings_all(&view), report(&doc["worlds"]["warnings"]))] {
        let (ours, theirs) = (without_retired(ours), without_retired(theirs));
        let keys: Vec<&String> = theirs.keys().chain(ours.keys()).collect();
        let diffs: Vec<String> = keys.iter().filter(|k| ours.get(**k) != theirs.get(**k)).map(|k| format!("{k}:\n  godot {:?}\n  rust  {:?}", theirs.get(*k), ours.get(*k))).collect();
        eprintln!("world {name}: {} godot keys, {} differ", theirs.len(), diffs.len());
        assert!(diffs.is_empty(), "world {name}:\n{}", diffs.join("\n"));
    }
}

/// The world lint recording: the same entries put into the same world (those Godot's upsert took), then every world's
/// errors and warnings.
#[test]
fn world_lint_matches_godot() {
    let Ok(text) = std::fs::read_to_string(root().join("games/emberwake/parity/out/checks.json")) else {
        eprintln!("no parity/out/checks.json: run games/emberwake/parity/checks.sh");
        return;
    };
    let doc: Value = serde_json::from_str(&text).unwrap();
    let lint = &doc["lint"];
    let src = source::load(&root().join("legacy/data"));
    let mut view = frozen_view(&src);
    let w = lint["world"].as_str().unwrap();
    for e in lint["entries"].as_array().unwrap() {
        if strings(&e["refused"]).is_empty() {
            check::world::put_entry(&mut view, w, e["kind"].as_str().unwrap(), e["entry"].as_object().unwrap());
        } else {
            eprintln!("godot refused {}: {:?}", e["entry"]["id"], strings(&e["refused"]));
        }
    }
    for (name, ours, theirs) in [("errors", check::world::validate_all(&view), report(&lint["errors"])), ("warnings", check::world::warnings_all(&view), report(&lint["warnings"]))] {
        let (ours, theirs) = (without_retired(ours), without_retired(theirs));
        let keys: Vec<&String> = theirs.keys().chain(ours.keys()).collect();
        let mut diffs: Vec<String> = Vec::new();
        for k in keys {
            if ours.get(k) != theirs.get(k) && !diffs.iter().any(|d| d.starts_with(&format!("{k}:"))) {
                let (g, r) = (theirs.get(k).cloned().unwrap_or_default(), ours.get(k).cloned().unwrap_or_default());
                let only_g: Vec<&String> = g.iter().filter(|m| !r.contains(m)).collect();
                let only_r: Vec<&String> = r.iter().filter(|m| !g.contains(m)).collect();
                diffs.push(format!("{k}:\n  godot only {only_g:?}\n  rust  only {only_r:?}\n  (same set, other order: {})", only_g.is_empty() && only_r.is_empty()));
            }
        }
        eprintln!("lint {name}: {} godot keys, {} differ", theirs.len(), diffs.len());
        assert!(diffs.is_empty(), "lint {name}:\n{}", diffs.join("\n"));
    }
}
