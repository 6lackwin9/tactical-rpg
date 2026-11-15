//! ASSETORG4 (FB-225): every portrait face sidecar (`*.face.json`) of the game's asset lineage (game, kit, engine
//! layers: `Layers::for_game`) uses the engine's canonical overlay names only. The engine dropped the read-time aliases
//! of the old Emberwake-era names, so an old name would now be refused (`parse_face`) or flagged as no overlay
//! (`face_problems`); this lint names the old spellings so a regression reads as what it is.

use aegis_core::assets::{face_problems, parse_face, Layers, IDLE_OVERLAYS};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The old overlay names and their canonical replacements (the sidecar key `veins` was `lines`).
const LEGACY: [(&str, &str); 4] = [("veins", "lines"), ("eye_glow", "point_glow"), ("ash", "particles_rise"), ("smoke", "drift")];

fn game_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn sidecars(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = read.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            sidecars(&p, out);
        } else if p.to_string_lossy().ends_with(".face.json") {
            out.push(p);
        }
    }
}

/// The legacy names one sidecar holds, as "path: old -> new" lines.
fn legacy_names(path: &Path, v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let Some(o) = v.as_object() else { return out };
    if o.contains_key("veins") {
        out.push(format!("{}: key 'veins' -> 'lines'", path.display()));
    }
    for e in o.get("idle").and_then(Value::as_array).into_iter().flatten() {
        let effect = e.get("effect").and_then(Value::as_str).unwrap_or("");
        if let Some((old, new)) = LEGACY.iter().find(|(old, _)| *old == effect) {
            out.push(format!("{}: idle effect '{old}' -> '{new}'", path.display()));
        }
    }
    out
}

#[test]
fn every_sidecar_of_the_lineage_uses_canonical_overlay_names() {
    let layers = Layers::for_game(&game_dir());
    assert!(layers.roots.len() >= 2, "the game and its kit layer: {:?}", layers.roots);
    let mut files = Vec::new();
    for root in &layers.roots {
        sidecars(root, &mut files);
    }
    assert!(!files.is_empty(), "the lineage ships face sidecars");
    let mut bad = Vec::new();
    for f in &files {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        bad.extend(legacy_names(f, &v));
        match parse_face(&v) {
            Err(p) => bad.push(format!("{}: {p:?}", f.display())),
            Ok(face) => bad.extend(face.idle.iter().filter(|o| !IDLE_OVERLAYS.contains(&o.effect.as_str())).map(|o| format!("{}: idle '{}' is not an overlay", f.display(), o.effect))),
        }
    }
    assert!(bad.is_empty(), "{} of {} sidecars hold legacy or unknown overlay names:\n{}", bad.len(), files.len(), bad.join("\n"));
}

#[test]
fn the_lint_and_the_engine_refuse_each_legacy_name() {
    let p = Path::new("x.face.json");
    for (old, new) in LEGACY {
        let v: Value = serde_json::json!({"size": 96, "idle": [{"effect": old}]});
        assert_eq!(legacy_names(p, &v), [format!("x.face.json: idle effect '{old}' -> '{new}'")]);
        let face = parse_face(&v).expect("an unknown effect name still parses");
        assert!(face_problems(&face, 96).iter().any(|m| m.contains(&format!("idle '{old}' is not an overlay"))), "{old}: the engine flags it");
    }
    let key = serde_json::json!({"size": 96, "veins": []});
    assert_eq!(legacy_names(p, &key), ["x.face.json: key 'veins' -> 'lines'"]);
    assert!(parse_face(&key).is_err(), "the old anchor key is refused by the engine");
}
