//! ENGINE-5: the trail's plan edits name what they edit as `subject`; the RPG kit's `unit` is read when it is missing,
//! so either trail flags the same churn.

use aegis_host::trail::build;
use serde_json::{json, Value};

fn churn(key: &str) -> Vec<Value> {
    (0..3).map(|i| json!({"t": 100 * i, "type": "plan_edit", "kind": "undo", key: "a1", "round": 2})).collect()
}

#[test]
fn a_subject_and_the_old_unit_flag_the_same_churn() {
    for key in ["subject", "unit"] {
        let report = build(&churn(key), 0, &[]);
        let flags: Vec<&Value> = report["flags"].as_array().unwrap().iter().filter(|f| f["kind"] == json!("undo_churn")).collect();
        assert_eq!(flags.len(), 1, "{key}: {report}");
        assert_eq!(flags[0]["detail"], json!("3 remove / undo / clear edits for a1 in round 2"), "{key}");
    }
}
