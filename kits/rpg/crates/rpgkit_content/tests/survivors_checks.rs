//! ACT1-HOOKS1 (FB-241 c), content side of the world encounter field `survivors` `{"flag", "unit"}`: the schema carries
//! it as a json field stored only when set (`{}` default, `omit_default`), so encounters written before it re-save byte
//! for byte; the check gives one error per broken rule: an object, known keys, a flag key the game may write (not a
//! reserved read key), guests to count, and a `unit` that is one of the guests (its ref, character id or base unit).

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{encounter, View};
use rpgkit_content::{source, store};
use serde_json::{json, Value};

fn with_ids(w: &str, rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = json::s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

fn view() -> View {
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {"characters": with_ids("w", json!([
        {"id": "cap_a", "name": "Cap A", "role": "npc", "base_unit": "captive"},
        {"id": "aide", "name": "Aide", "role": "npc", "base_unit": "squire"}
    ]))}}});
    View::new(&json!({}), &worlds, &json!({}), Default::default())
}

fn enc(survivors: Value, guests: Value) -> Dict {
    json::dict(&json!({"id": "rescue", "name": "Rescue", "guests": guests, "survivors": survivors})).clone()
}

fn problems(survivors: Value) -> Vec<String> {
    encounter::survivors_problems(&view(), "w", &enc(survivors, json!(["cap_a", "aide"])))
}

#[test]
fn the_field_is_stored_only_when_set() {
    let f = fields(source::schema(), "world", "encounters").iter().map(json::dict).find(|f| json::s(f, "key") == "survivors").cloned().expect("the schema field");
    assert_eq!((json::s(&f, "type"), json::get(&f, "default").clone()), ("json".to_string(), json!({})));
    assert!(json::b(&f, "omit_default", false), "stored only when set");
    let plain = json::dict(&json!({"id": "rescue", "name": "Rescue", "enemies": ["rat"]})).clone();
    assert!(!store::world_storage("encounters", &plain).contains_key("survivors"), "a row without it re-saves without it");
    let set = enc(json!({"flag": "w.saved", "unit": "captive"}), json!(["cap_a"]));
    assert_eq!(store::world_storage("encounters", &set).get("survivors"), set.get("survivors"), "a set block is written");
}

#[test]
fn a_valid_block_passes() {
    assert!(problems(json!({})).is_empty(), "{{}}: none");
    for unit in ["", "captive", "cap_a", "aide", "squire"] {
        assert!(problems(json!({"flag": "w.saved", "unit": unit})).is_empty(), "unit '{unit}': a guest's ref, character or base unit");
    }
    assert!(problems(json!({"flag": "w.saved"})).is_empty(), "unit left out: every guest");
    assert!(encounter::survivors_problems(&view(), "w", &json::dict(&json!({"id": "x", "guests": []})).clone()).is_empty(), "no field: none");
}

#[test]
fn a_bad_block_names_each_problem() {
    assert_eq!(problems(json!([])), ["Survivors must be an object {flag, unit}."]);
    assert_eq!(problems(json!({"flag": "w.saved", "count": 2})), ["Survivors: unknown key 'count'."]);
    assert_eq!(problems(json!({"unit": "captive"})), ["Survivors flag '' must be a flag key (a-z, 0-9, _ and .)."]);
    assert_eq!(problems(json!({"flag": "W Saved"})), ["Survivors flag 'W Saved' must be a flag key (a-z, 0-9, _ and .)."]);
    let reserved = problems(json!({"flag": "campaign.saved"}));
    assert!(reserved.len() == 1 && reserved[0].starts_with("Survivors: flag key 'campaign.saved' is reserved"), "{reserved:?}");
    assert_eq!(problems(json!({"flag": "w.saved", "unit": "rat"})), ["Survivors unit 'rat' is none of the guests (a guest's character id or base unit)."]);
    let lonely = encounter::survivors_problems(&view(), "w", &enc(json!({"flag": "w.saved"}), json!([])));
    assert_eq!(lonely, ["Survivors needs the encounter's guests (it counts the guests that lived)."]);
}
