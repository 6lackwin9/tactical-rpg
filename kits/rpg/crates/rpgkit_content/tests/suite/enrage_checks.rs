//! ENRAGE2 (PR7-55, `docs/specs/ENRAGE2.md` EN4, EN12): the ruleset row's baseline enrage numbers and the encounter's
//! `escaped` timeout outcome, on rows built here (no shipped data). The row reads its numbers (none: no enrage, so other
//! kit users are unchanged); `enrage_step` names unit tiers with whole % 0-100; the writer stores the numbers only off
//! their defaults; the schema offers `escaped` and the battle reads it.

use aegis_core::json;
use rpgkit_content::store;
use rpgkit_rules::ruleset::{self, Numbers};
use serde_json::{json, Value};

fn row(extra: Value) -> aegis_core::json::Dict {
    let mut r = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent"]});
    for (k, v) in extra.as_object().unwrap() {
        r[k] = v.clone();
    }
    r.as_object().cloned().unwrap()
}

#[test]
fn a_row_reads_its_enrage_numbers_and_none_means_off() {
    let n = Numbers::from_row(&row(json!({"enrage_from_round": 6, "enrage_step": {"minion": 10, "boss": 20}, "enrage_max_stacks": 3})));
    assert_eq!((n.enrage_power("boss", 5), n.enrage_power("boss", 6), n.enrage_power("boss", 7)), (0, 20, 40));
    assert_eq!(n.enrage_power("boss", 20), 60, "at most 3 steps");
    assert_eq!((n.enrage_power("minion", 6), n.enrage_power("elite", 9), n.enrage_power("hero", 9)), (10, 0, 0), "a tier not named gains none");
    let off = Numbers::from_row(&row(json!({})));
    assert_eq!((off.enrage_from_round, off.enrage_power("boss", 12)), (0, 0));
    assert_eq!(off.field("enrage_max_stacks").as_deref(), Some("4"));
}

#[test]
fn the_enrage_step_names_unit_tiers_with_whole_percents() {
    let errors = |step: Value| {
        let t = aegis_core::scratch::Scratch::vacant("rpgkit_enrage_checks_step");
        let data = t.join("data");
        std::fs::create_dir_all(data.join("catalog")).unwrap();
        std::fs::create_dir_all(data.join("worlds")).unwrap();
        let rows = json!({"kind": "rulesets", "version": 1, "entries": [Value::Object(row(json!({"enrage_step": step})))]});
        std::fs::write(data.join("catalog/rulesets.json"), rows.to_string()).unwrap();
        store::check(&data)
    };
    assert!(errors(json!({"minion": 10, "standard": 10, "elite": 15, "boss": 20})).iter().all(|e| !e.contains("Enrage")));
    assert!(errors(json!({"champion": 10})).iter().any(|e| e.contains("'champion' is not a unit tier")));
    assert!(errors(json!({"boss": 12.5})).iter().any(|e| e.contains("boss must be a whole % 0-100")));
    assert!(errors(json!({"boss": 101})).iter().any(|e| e.contains("boss must be a whole % 0-100")));
    assert!(errors(json!([10])).iter().any(|e| e.contains("Enrage step by tier must be an object")));
}

#[test]
fn the_writer_stores_the_enrage_numbers_only_when_set() {
    let stored = store::catalog_storage(ruleset::KIND, &row(json!({"enrage_from_round": 0, "enrage_step": {}, "enrage_max_stacks": 4})), &[]).unwrap();
    for key in store::RULESET_ENRAGE_FIELDS {
        assert!(!stored.contains_key(key), "{key} at its default is no key");
    }
    let set = store::catalog_storage(ruleset::KIND, &row(json!({"enrage_from_round": 6, "enrage_step": {"boss": 20}})), &[]).unwrap();
    assert_eq!((json::get(&set, "enrage_from_round"), json::get(&set, "enrage_step")), (&json!(6), &json!({"boss": 20})));
    assert!(!set.contains_key("enrage_max_stacks"));
}

#[test]
fn the_schema_offers_escaped_and_the_battle_reads_it() {
    for section in ["content", "world"] {
        let f = rpgkit_content::source::schema()[section]["encounters"].as_array().unwrap().iter().find(|f| f["key"] == "timeout_outcome").cloned().unwrap();
        assert_eq!(f["options"], json!(["fled", "lost", "escaped"]), "{section}");
    }
    let o = json!({"timeout_outcome": "escaped"});
    assert_eq!(rpgkit_battle::encounter::timeout_outcome(o.as_object().unwrap()), rpgkit_battle::encounter::ESCAPED);
}
