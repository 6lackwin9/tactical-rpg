//! T1.2-S1 (FB-54a): the dialogue condition check takes `threshold` on a `stat` condition as a pure alias of `min`: a
//! valid bar passes either way, a bad bar gets the same message naming the key used, both keys on one condition are
//! refused, and `threshold` stays a `stat`-only key (a `gold` condition refuses it as unknown).

use rpgkit_content::check::{dialogue, View};
use serde_json::{json, Value};

const LABEL: &str = "Requires";

fn view() -> View {
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {}}});
    View::new(&json!({}), &worlds, &json!({}), Default::default())
}

fn errors(conds: Value) -> Vec<String> {
    dialogue::conditions_errors(&view(), "w", &conds, LABEL)
}

#[test]
fn a_threshold_bar_is_checked_like_a_min_bar() {
    for bar in [json!(5), json!(0), json!(1000), json!(2.5), json!("x")] {
        let by_min = errors(json!([{"kind": "stat", "stat": "STR", "min": bar}]));
        let by_threshold = errors(json!([{"kind": "stat", "stat": "STR", "threshold": bar}]));
        let renamed: Vec<String> = by_min.iter().map(|e| e.replace(": min must", ": threshold must")).collect();
        assert_eq!(by_threshold, renamed, "bar {bar}: the same verdict, naming the key used");
    }
    assert!(errors(json!([{"kind": "stat", "stat": "STR", "threshold": 5}])).is_empty(), "a valid threshold passes");
}

#[test]
fn min_and_threshold_together_are_refused() {
    let got = errors(json!([{"kind": "stat", "stat": "STR", "min": 5, "threshold": 5}]));
    assert_eq!(got, [format!("{LABEL} 1 (stat): give min or threshold (its alias), not both.")]);
}

#[test]
fn threshold_is_a_stat_key_only() {
    let got = errors(json!([{"kind": "gold", "min": 5, "threshold": 5}]));
    assert_eq!(got, [format!("{LABEL} 1 (gold): unknown key 'threshold'.")]);
}
