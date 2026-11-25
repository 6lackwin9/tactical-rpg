//! PR7-1 on the shipped data, read only: no `sunk_ward` meter, faction, quest or loot text carries a `PLACEHOLDER`
//! marker, no quest reward text names "gold" (whole word, case-insensitive; data keys are out of scope), and quest
//! `q_a1_m5` is named "The Crucible". Every string value of every row is walked, so no field name is hard-coded
//! except the quest `reward` text and `name`.

use serde_json::Value;
use std::path::Path;

fn rows(file: &str) -> Vec<Value> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/worlds/sunk_ward").join(file);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&root).expect("read")).expect("json");
    v["entries"].as_array().expect("entries").clone()
}

fn strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| strings(x, out)),
        _ => {}
    }
}

fn has_word(text: &str, word: &str) -> bool {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w == word)
}

#[test]
fn no_placeholder_text() {
    for file in ["meters.json", "factions.json", "quests.json", "loot.json"] {
        for row in rows(file) {
            let mut all = Vec::new();
            strings(&row, &mut all);
            for s in all {
                assert!(!s.to_uppercase().contains("PLACEHOLDER"), "{file} {}: {s}", row["id"]);
            }
        }
    }
}

#[test]
fn quest_rewards_do_not_say_gold() {
    for q in rows("quests.json") {
        let mut all = Vec::new();
        strings(&q["reward"], &mut all);
        for s in all {
            assert!(!has_word(&s, "gold"), "quest {}: {s}", q["id"]);
        }
    }
}

#[test]
fn crucible_quest_name() {
    let q = rows("quests.json").into_iter().find(|q| q["id"] == "q_a1_m5").expect("q_a1_m5");
    assert_eq!(q["name"], "The Crucible");
}
