//! Data independence: the engine's own loader (emberwake_content::source) reads a data folder and must build exactly what
//! Godot's catalogs build from it, as recorded by engine/parity/export.sh from legacy/ (so the loader reads
//! legacy/data): every catalog kind (skill ranks expanded, items resolved against their families), every world (manifest, lineage, own entries) and the game data
//! (maps, quests, services, XP curve, item families). Keys, key order and int / float types must match; floats match
//! to 1e-9 (Godot's JSON.stringify writes fewer digits than it computes with). The schema copy must match the export.

use emberwake_content::source;
use serde_json::Value;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/parity/out")
}

fn data_dir() -> PathBuf {
    // the data Godot read for the exports: the frozen copy (the live data/ is checked by tests/store.rs)
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../legacy/data")
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
    let Some(catalog) = read("catalog.json") else {
        eprintln!("no parity/out/catalog.json: run engine/parity/export.sh");
        return;
    };
    let src = source::load(&data_dir());
    assert!(src.errors.is_empty(), "load errors: {:?}", src.errors);
    let mut problems = Vec::new();
    for kind in source::CATALOG_KINDS {
        problems.push(diff(&format!("catalog.{kind}"), &catalog[kind], &src.catalog[kind]));
    }
    let worlds = read("worlds.json").unwrap();
    problems.push(diff("worlds", &worlds, &src.worlds));
    let game = read("game_data.json").unwrap();
    for key in ["maps", "quests", "services", "xp_curve", "xp_for_level", "item_families"] {
        problems.push(diff(&format!("game.{key}"), &game[key], &src.game[key]));
    }
    if let Some(schema) = read("schema.json") {
        problems.push(diff("schema", &schema, source::schema()));
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
