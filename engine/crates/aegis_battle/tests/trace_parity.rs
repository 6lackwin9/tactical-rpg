//! Step-by-step parity with Godot battle traces (engine/parity/godot/trace.gd). The RNG, sorts and dictionary order
//! are reproduced exactly, so a seeded duel should match step for step; the first divergence is reported.

use aegis_battle::sim::{fight, FightSpec};
use aegis_core::value;
use aegis_data::{Catalog, UnitDef};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out")
}

fn load(name: &str) -> Value {
    let p = out_dir().join(name);
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|_| panic!("missing {} - run engine/parity/trace.sh", p.display()))).unwrap()
}

/// First difference between the Godot trace and the Rust one, as a readable line.
fn first_divergence(g: &Value, r: &Value) -> Option<String> {
    if g["start"] != r["start"] {
        return Some(format!("start: godot {} rust {}", g["start"], r["start"]));
    }
    let (gr, rr) = (g["rounds"].as_array().unwrap(), r["rounds"].as_array().unwrap());
    for (i, (a, b)) in gr.iter().zip(rr.iter()).enumerate() {
        if a["actions"] != b["actions"] {
            return Some(format!("round {} actions:\n  godot {}\n  rust  {}", i + 1, a["actions"], b["actions"]));
        }
        let (gs, rs) = (a["steps"].as_array().unwrap(), b["steps"].as_array().unwrap());
        for (j, (x, y)) in gs.iter().zip(rs.iter()).enumerate() {
            if x != y {
                let prev = if j > 0 { gs[j - 1].to_string() } else { "-".into() };
                return Some(format!("round {} step {}:\n  before {}\n  godot  {}\n  rust   {}", i + 1, j + 1, prev, x, y));
            }
        }
        if gs.len() != rs.len() {
            return Some(format!("round {} step count: godot {} rust {}", i + 1, gs.len(), rs.len()));
        }
    }
    if gr.len() != rr.len() {
        return Some(format!("round count: godot {} rust {}", gr.len(), rr.len()));
    }
    None
}

#[test]
fn battles_replay_godot_step_for_step() {
    let cat = Arc::new(Catalog::from_value(&load("catalog.json")));
    let mut files: Vec<String> = std::fs::read_dir(out_dir()).expect("missing engine/parity/out").filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.starts_with("traces") && !n.starts_with("traces_world") && n.ends_with(".json")).collect();
    files.sort();
    if files.is_empty() {
        // a fresh checkout (CI records only the exports): the traces come from engine/parity/trace.sh (tools/fc.sh parity)
        eprintln!("no traces*.json: run engine/parity/trace.sh");
        return;
    }
    let mut total = 0;
    for name in files {
        total += replay_file(&cat, &name);
    }
    eprintln!("{total} fights match Godot step for step");
}

fn replay_file(cat: &Arc<Catalog>, name: &str) -> usize {
    let t = load(name);
    let units: IndexMap<String, UnitDef> = t["units"].as_object().unwrap().iter().map(|(k, v)| (k.clone(), UnitDef::from_obj(v.as_object().unwrap()))).collect();
    let jitter = t["jitter"].as_str() == Some("on");
    let pouch: Vec<String> = t["pouch"].as_str().unwrap_or("").split(',').filter(|s| !s.is_empty()).map(String::from).collect();
    let fights = t["fights"].as_array().unwrap();
    let mut failures = Vec::new();
    for f in fights {
        let fo = f.as_object().unwrap();
        let spec = FightSpec { left: value::strings(fo, "left"), right: value::strings(fo, "right"), seed: f["seed"].as_i64().unwrap(), preset: value::string(fo, "preset", "balanced"), cap: 10, jitter, base: f.get("base").and_then(|b| b.as_object()).filter(|b| !b.is_empty()).cloned(), pouch: pouch.clone() };
        let res = fight(cat.clone(), &units, &spec, true);
        if let Some(d) = first_divergence(f, res.trace.as_ref().unwrap()) {
            failures.push(format!("{:?} vs {:?} seed {}: {d}", spec.left, spec.right, spec.seed));
        }
    }
    if !failures.is_empty() {
        panic!("{name}: {} of {} fights diverge; first:\n{}", failures.len(), fights.len(), failures[0]);
    }
    eprintln!("{name}: {} fights", fights.len());
    fights.len()
}
