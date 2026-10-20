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
    let art = doc["art_keys"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), strings(v))).collect()).unwrap_or_default();
    let view = View::new(&src.catalog, &src.worlds, &src.game, art);
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
        let (errors, warnings) = (check::validate_entry(&view, kind, entry, current), check::warnings_for(&view, kind, entry));
        let ok = errors == strings(&c["errors"]) && warnings == strings(&c["warnings"]);
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
    let art = doc["art_keys"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), strings(v))).collect()).unwrap_or_default();
    assert_eq!(emberwake_content::source::art_keys(&root().join("legacy/data")), art, "art keys = AssetRegistry.keys");
    let mut view = View::new(&src.catalog, &src.worlds, &src.game, art);
    view.terrain_file = root().join("legacy/data/catalog/terrain.json").exists();
    for (name, ours, theirs) in [("errors", check::validate_all(&view), report(&doc["catalog"]["errors"])), ("warnings", check::warnings_all(&view), report(&doc["catalog"]["warnings"]))] {
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
    let art = doc["art_keys"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), strings(v))).collect()).unwrap_or_default();
    let view = View::new(&src.catalog, &src.worlds, &src.game, art);
    let show: usize = std::env::var("CHECKS_SHOW").ok().and_then(|s| s.parse().ok()).unwrap_or(2);
    let mut per_kind: indexmap::IndexMap<String, (usize, usize)> = indexmap::IndexMap::new();
    let mut failures = Vec::new();
    for c in doc["cases"].as_array().unwrap().iter().filter(|c| c["world"] != "") {
        let (w, kind) = (c["world"].as_str().unwrap(), c["kind"].as_str().unwrap());
        let (entry, current) = (c["entry"].as_object().unwrap(), c["current"].as_str().unwrap());
        let errors = check::world::validate_entry(&view, w, kind, entry, current);
        let blocking = check::world::blocking_errors(&view, w, kind, entry, current);
        let slot = per_kind.entry(kind.to_string()).or_default();
        slot.0 += 1;
        if errors != strings(&c["errors"]) || blocking != strings(&c["blocking"]) {
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
    let art = doc["art_keys"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), strings(v))).collect()).unwrap_or_default();
    let mut view = View::new(&src.catalog, &src.worlds, &src.game, art);
    let w = lint["world"].as_str().unwrap();
    for e in lint["entries"].as_array().unwrap() {
        if strings(&e["refused"]).is_empty() {
            check::world::put_entry(&mut view, w, e["kind"].as_str().unwrap(), e["entry"].as_object().unwrap());
        } else {
            eprintln!("godot refused {}: {:?}", e["entry"]["id"], strings(&e["refused"]));
        }
    }
    for (name, ours, theirs) in [("errors", check::world::validate_all(&view), report(&lint["errors"])), ("warnings", check::world::warnings_all(&view), report(&lint["warnings"]))] {
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
