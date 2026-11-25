//! PR5-26b (`FB-198` B9 Option B): a ranked skill entry's rank modifier may only hold rank fields (the schema's
//! `"rank": true`) and its rank's `description`; any other key (e.g. `hp_cost_pct`, `uses_per_battle`, both
//! `"rank": false`) is one check error naming the skill, the rank and the key, instead of dropping silently: the loader
//! reports it (the entry still loads) and the writer refuses an entry carrying one. On a scratch data folder built here
//! (no shipped data); the keys come from the schema.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{source, store};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// The phrase every non-rank key error carries.
const NOT_RANK: &str = "which is not a rank field";
/// The keys a modifier refuses with their own reason (rank names and ids are derived) and the per-rank text.
const OWN_REASON: [&str; 5] = ["name", "family", "tier", "id", "description"];

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_rank_modifier_keys_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

/// Engine skills and a Jab with one rank modifier `rank2`.
fn catalog(data: &Path, rank2: Value) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(
        data,
        "skills",
        2,
        json!([
            one("move", "MOVE", json!({"system": true})),
            one("rest", "REST", json!({"system": true})),
            one("strike", "ATTACK", json!({"basic": true})),
            one("jab", "ATTACK", json!({"ranks": [rank2]}))
        ]),
    );
}

fn not_rank_errors(data: &Path) -> Vec<String> {
    store::check(data).into_iter().filter(|e| e.contains(NOT_RANK)).collect()
}

fn dict(v: Value) -> Dict {
    v.as_object().unwrap().clone()
}

#[test]
fn a_non_rank_key_in_a_rank_modifier_is_one_check_error() {
    let (_t, data) = scratch("check");
    catalog(&data, json!({"power_mult": 1.5, "hp_cost_pct": 10, "uses_per_battle": 2}));
    let errors = not_rank_errors(&data);
    assert_eq!(errors.len(), 2, "{errors:?}");
    for key in ["hp_cost_pct", "uses_per_battle"] {
        assert!(errors.iter().any(|e| e.contains("skill 'jab' has a rank 2 modifier with") && e.contains(&format!("'{key}'"))), "{key}: {errors:?}");
    }
    // the entry still loads (its rank fields as before)
    let v = store::view(&data);
    assert!(v.has("skills", "jab_2"));
    assert_eq!(json::f(v.get("skills", "jab_2"), "power_mult", 0.0), 1.5);
    // rank fields only: clean
    let (_t, data) = scratch("clean");
    catalog(&data, json!({"power_mult": 1.5, "description": "Jab II"}));
    assert_eq!(not_rank_errors(&data), Vec::<String>::new());
}

/// Every schema skill field: one with `"rank": true` is accepted in a modifier, one with `"rank": false` is refused
/// (the derived keys and the description keep their own rules).
#[test]
fn every_schema_field_follows_its_rank_flag() {
    for f in fields(source::schema(), "content", "skills") {
        let f = json::dict(f);
        let key = json::s(f, "key");
        if OWN_REASON.contains(&key.as_str()) {
            continue;
        }
        let (_t, data) = scratch(&format!("field_{key}"));
        catalog(&data, json!({ key.clone(): f.get("default").cloned().unwrap_or(Value::Null) }));
        let errors = not_rank_errors(&data);
        if json::b(f, "rank", false) {
            assert_eq!(errors, Vec::<String>::new(), "rank field '{key}'");
        } else {
            assert_eq!(errors.len(), 1, "non-rank field '{key}': {errors:?}");
            assert!(errors[0].contains(&format!("'{key}'")), "{errors:?}");
        }
    }
}

/// The writer refuses an entry whose modifier holds a non-rank key (the file is unchanged); the same entry with rank
/// fields only is written.
#[test]
fn the_writer_refuses_a_non_rank_key_in_a_modifier() {
    let (_t, data) = scratch("writer");
    catalog(&data, json!({"power_mult": 1.5}));
    let file = data.join("catalog/skills.json");
    let before = fs::read_to_string(&file).unwrap();
    let entry = |rank2: Value| dict(json!({"id": "jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "ranks": [rank2]}));
    let refused = store::upsert(&data, None, "skills", &entry(json!({"power_mult": 2.0, "hp_cost_pct": 10})), Some("jab")).err().expect("refused");
    assert!(refused.iter().any(|e| e.contains(NOT_RANK) && e.contains("'hp_cost_pct'")), "{refused:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), before, "nothing written");
    store::upsert(&data, None, "skills", &entry(json!({"power_mult": 2.0})), Some("jab")).expect("rank fields only");
    let v = store::view(&data);
    assert_eq!(json::f(v.get("skills", "jab_2"), "power_mult", 0.0), 2.0);
}
