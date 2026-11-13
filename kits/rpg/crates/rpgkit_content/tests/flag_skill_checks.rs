//! ACT1-HOOKS2 (FB-244 (1)), content side of the world encounter field `flag_skills`
//! `[{"member_from_flag", "skill", "requires_flags"}]`: the schema carries it as a json field stored only when set (`[]`
//! default, `omit_default`), so encounters written before it re-save byte for byte; the check gives one error per
//! broken rule: a list of at most `FLAG_SKILLS_MAX` objects, known keys, a flag key, a known catalog skill that is not
//! a system skill, and `requires_flags` conditions.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{encounter, View};
use rpgkit_content::{source, store};
use serde_json::{json, Value};

fn view() -> View {
    let catalog = json!({"skills": [
        {"id": "s_grant", "name": "Grant", "kind": "ATTACK", "level": 1, "blocks": 1},
        {"id": "s_move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}
    ]});
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {}}});
    View::new(&catalog, &worlds, &json!({}), Default::default())
}

fn enc(grants: Value) -> Dict {
    json::dict(&json!({"id": "trial", "name": "Trial", "enemies": ["foe"], "flag_skills": grants})).clone()
}

fn problems(grants: Value) -> Vec<String> {
    encounter::flag_skills_problems(&view(), &enc(grants))
}

fn grant(skill: &str) -> Value {
    json!({"member_from_flag": "w.chosen", "skill": skill})
}

#[test]
fn the_field_is_stored_only_when_set() {
    let f = fields(source::schema(), "world", "encounters").iter().map(json::dict).find(|f| json::s(f, "key") == "flag_skills").cloned().expect("the schema field");
    assert_eq!((json::s(&f, "type"), json::get(&f, "default").clone()), ("json".to_string(), json!([])));
    assert!(json::b(&f, "omit_default", false), "stored only when set");
    let plain = json::dict(&json!({"id": "trial", "name": "Trial", "enemies": ["foe"]})).clone();
    assert!(!store::world_storage("encounters", &plain).contains_key("flag_skills"), "a row without it re-saves without it");
    let set = enc(json!([grant("s_grant")]));
    assert_eq!(store::world_storage("encounters", &set).get("flag_skills"), set.get("flag_skills"), "a set list is written");
}

#[test]
fn a_valid_list_passes() {
    assert!(problems(json!([])).is_empty(), "[]: none");
    assert!(encounter::flag_skills_problems(&view(), &json::dict(&json!({"id": "x"})).clone()).is_empty(), "no field: none");
    let gated = json!([{"member_from_flag": "w.chosen", "skill": "s_grant", "requires_flags": [{"key": "w.ready", "op": "==", "value": true}]}]);
    assert!(problems(gated).is_empty());
    assert!(problems(Value::Array(vec![grant("s_grant"); rpgkit_data::FLAG_SKILLS_MAX])).is_empty(), "the most entries");
}

#[test]
fn a_bad_list_names_each_problem() {
    assert_eq!(problems(json!({})), ["Flag skills must be a list of {member_from_flag, skill, requires_flags}."]);
    let most = rpgkit_data::FLAG_SKILLS_MAX;
    assert_eq!(problems(Value::Array(vec![grant("s_grant"); most + 1])), [format!("Flag skills holds at most {most} entries (has {}).", most + 1)]);
    assert_eq!(problems(json!(["s_grant"])), ["Flag skill 1 must be an object {member_from_flag, skill, requires_flags}."]);
    assert_eq!(problems(json!([{"member_from_flag": "w.chosen", "skill": "s_grant", "rounds": 2}])), ["Flag skill 1: unknown key 'rounds'."]);
    assert_eq!(problems(json!([{"skill": "s_grant"}])), ["Flag skill 1: member_from_flag '' must be a flag key (a-z, 0-9, _ and .)."]);
    assert_eq!(problems(json!([grant("s_nope")])), ["Flag skill 1: unknown skill 's_nope'."]);
    assert_eq!(problems(json!([grant("s_move")])), ["Flag skill 1: 's_move' is a system skill (every unit has it)."]);
    let bad_cond = problems(json!([{"member_from_flag": "w.chosen", "skill": "s_grant", "requires_flags": [{"key": "W A"}]}]));
    assert_eq!(bad_cond, ["Flag skill 1: requires_flags 1: flag key 'W A' must be non-empty [a-z0-9_.]."]);
}
