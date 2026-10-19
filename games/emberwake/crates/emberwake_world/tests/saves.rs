//! Godot save files load unchanged: every save fixture (scripts/tests/fixtures/saves, one per save version) is
//! decoded by the engine (migrations included) and must give Godot's result (parity/out/saves.json from
//! `engine/parity/export.sh only=saves`): ok, errors, warnings and the loaded state. A save the engine writes must also
//! read back to the same state.

use aegis_core::json;
use emberwake_world::{save, Data};
use serde_json::Value;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../engine/parity/out")
}

#[test]
fn godot_saves_load_the_same() {
    let dir = out_dir();
    let path = dir.join("saves.json");
    if !path.exists() {
        eprintln!("no {}: run engine/parity/export.sh first", path.display());
        return;
    }
    let d = Data::load(&dir).expect("parity exports");
    let rows: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut failures = Vec::new();
    for row in json::arr(&rows) {
        let row = json::dict(row);
        let file = json::s(row, "file");
        let res = save::decode(&d, &json::s(row, "text"));
        let godot_ok = json::b(row, "ok", false);
        match res {
            Ok(l) => {
                if !godot_ok {
                    failures.push(format!("{file}: engine loads it, Godot refuses {:?}", json::sget(row, "errors")));
                    continue;
                }
                if l.warnings != json::sget(row, "warnings") {
                    failures.push(format!("{file}: warnings {:?} vs Godot {:?}", l.warnings, json::sget(row, "warnings")));
                }
                if !json::deep_equal(&l.state.to_value(), json::get(row, "state")) {
                    failures.push(format!("{file}: state differs\n  engine {}\n  godot  {}", l.state.to_value(), json::get(row, "state")));
                }
                // Round trip through an engine save.
                let tmp = std::env::temp_dir().join(format!("aegis_save_test_{}", std::process::id()));
                let errors = save::write(&d, &tmp, &l.state, 1, 1);
                let back = save::read(&d, &tmp, 1);
                let _ = std::fs::remove_dir_all(&tmp);
                match back {
                    Ok(b) if errors.is_empty() && b.state == l.state => {}
                    Ok(_) => failures.push(format!("{file}: engine write/read round trip changed the state ({errors:?})")),
                    Err(e) => failures.push(format!("{file}: engine save does not read back: {e:?}")),
                }
            }
            Err(e) => {
                if godot_ok || e != json::sget(row, "errors") {
                    failures.push(format!("{file}: engine refuses {e:?}, Godot ok={godot_ok} {:?}", json::sget(row, "errors")));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
