//! S1.2-S4 (docs/sessions/S1_STATUS.md section 1, multi-round guard): `guard_rounds` is a row-only skill key until its
//! schema entry: it loads into the typed catalog (1 when absent), survives the loader, and the row check allows it on a
//! DEFEND skill at 1-5 only. Scratch data folders only.

use aegis_core::json::Dict;
use rpgkit_content::check;
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn scratch() -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant("rpgkit_guard_rows");
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let skills = json!([
        {"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.2, "guard_rounds": 3, "description": "Braces.", "ranks": []},
        {"id": "t_guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3, "description": "Guards.", "ranks": []},
        {"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "description": "Jabs.", "ranks": []},
    ]);
    fs::write(data.join("catalog/skills.json"), json!({"kind": "skills", "version": 2, "entries": skills}).to_string()).unwrap();
    (t, data)
}

fn guard_errors(v: &check::View, id: &str, change: Value) -> Vec<String> {
    let mut e: Dict = v.get("skills", id).clone();
    for (k, x) in change.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, "skills", &e, id).into_iter().filter(|m| m.contains("Guard rounds")).collect()
}

#[test]
fn guard_rounds_loads_and_is_checked_on_defend_rows() {
    let (_t, data) = scratch();
    let src = source::load(&data);
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let rounds = |id: &str| cat.skills.iter().find(|s| s.id == id).map(|s| s.guard_rounds);
    assert_eq!([rounds("t_brace"), rounds("t_guard"), rounds("t_jab")], [Some(3), Some(1), Some(1)], "the loader keeps the row-only key; 1 when absent");
    let v = store::view(&data);
    assert!(guard_errors(&v, "t_brace", json!({})).is_empty());
    assert!(guard_errors(&v, "t_guard", json!({"guard_rounds": 5})).is_empty());
    assert_eq!(guard_errors(&v, "t_guard", json!({"guard_rounds": 6})), ["Guard rounds must be 1-5."]);
    assert_eq!(guard_errors(&v, "t_guard", json!({"guard_rounds": 0})), ["Guard rounds must be 1-5."]);
    assert_eq!(guard_errors(&v, "t_jab", json!({"guard_rounds": 2})), ["Guard rounds belong to DEFEND skills, not ATTACK."]);
}
