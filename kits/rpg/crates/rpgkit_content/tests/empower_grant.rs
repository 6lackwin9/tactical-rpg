//! EMPOWER1: the `empower` row carries the support tag, so a skill may grant it through `grant_status` or
//! `extra_grants` inside the row's magnitude range; the numbers come from the row. Scratch data folder only.

use aegis_core::json::Dict;
use rpgkit_content::{check, store};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::fs;

fn errors(v: &check::View, id: &str, change: Value) -> Vec<String> {
    let base = if id == "aid" {
        json!({"id": "aid", "name": "Aid", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY", "heal_base": 5, "description": "Aids."})
    } else {
        json!({"id": "stand", "name": "Stand", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.2, "description": "Stands."})
    };
    let mut e: Dict = base.as_object().unwrap().clone();
    e.extend(change.as_object().unwrap().clone());
    check::validate_entry(v, "skills", &e, id)
}

#[test]
fn a_skill_grants_empower_inside_its_range() {
    let row = status::row_of("empower").unwrap();
    assert!(status::buffs().contains(&"empower") && row.grant == "support" && !status::intent_bound("empower"));
    let (lo, hi) = (row.magnitude_min, row.magnitude_max);

    let root = aegis_core::scratch::Scratch::vacant("rpgkit_empower_grant");
    let _ = fs::remove_dir_all(&root);
    let data = root.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    let v = store::view(&data);

    for id in ["aid", "stand"] {
        for m in [lo, hi] {
            let got = errors(&v, id, json!({"grant_status": "empower", "status_magnitude": m, "status_rounds": 1}));
            assert!(got.is_empty(), "{id} grant {m}: {got:?}");
        }
        let over = errors(&v, id, json!({"grant_status": "empower", "status_magnitude": hi + 1, "status_rounds": 1}));
        assert_eq!(over.len(), 1, "{id} grant over: {over:?}");
        assert!(over[0].contains("agnitude"), "{over:?}");

        for m in [lo, hi] {
            let got = errors(&v, id, json!({"extra_grants": [{"status": "empower", "rounds": 1, "magnitude": m}]}));
            assert!(got.is_empty(), "{id} extra {m}: {got:?}");
        }
        for m in [lo - 1, hi + 1] {
            let got = errors(&v, id, json!({"extra_grants": [{"status": "empower", "rounds": 1, "magnitude": m}]}));
            assert_eq!(got, vec![format!("Extra grant 'empower' needs a magnitude {lo}-{hi}.")], "{id} extra {m}");
        }
    }
    let _ = fs::remove_dir_all(&root);
}
