//! BUNKS-UPGRADE-PROPS1 (`PR7-50`): a map prop's `until_bunk` (the bunk prop whose place it holds until the bunk cap
//! reaches it): the checks take it as a known key, refuse one that is not text or names no bunk prop of the map, refuse
//! it on a bunk prop, and let the prop share the tiles of the bunk it holds (never another's); the writer leaves it out
//! when "". Test-only maps; the expectations follow from the rules.

use aegis_core::json::Dict;
use rpgkit_content::check::map;
use rpgkit_content::source;
use serde_json::{json, Value};

/// 10 x 5: walls round the edge, a 3 x 1 wall strip at x 3-5, y 3 for the bunk and what holds its place.
const ROWS: [&str; 5] = ["##########", "#........#", "#........#", "#..###...#", "##########"];

fn template(props: Value) -> Dict {
    let t = json!({"id": "test_map", "name": "T", "rows": ROWS, "anchors": [], "paths": {}, "props": props, "tags": []});
    let mut t = t.as_object().cloned().unwrap();
    if let Some(Value::Array(list)) = t.get("props").cloned() {
        t.insert("props".into(), Value::Array(list.iter().map(|p| Value::Object(source::prop_full(p.as_object().unwrap()))).collect()));
    }
    t
}

fn bunk() -> Value {
    json!({"id": "bunk_7", "art": "cot", "x": 3, "y": 3, "w": 3, "h": 1})
}

fn held(id: &str, x: i64, w: i64, until: Value) -> Value {
    json!({"id": id, "art": "crate_stack", "x": x, "y": 3, "w": w, "h": 1, "until_bunk": until})
}

#[test]
fn a_prop_holding_a_bunks_place_shares_its_tiles() {
    assert_eq!(map::prop_errors(&template(json!([bunk(), held("spare_timber", 3, 2, json!("bunk_7")), held("spare_pallet", 5, 1, json!("bunk_7"))]))), Vec::<String>::new());
}

#[test]
fn until_bunk_names_a_bunk_prop_of_the_map_and_a_bunk_takes_none() {
    let errors = |props: Value| map::prop_errors(&template(props));
    let not_text = errors(json!([bunk(), held("spare", 3, 2, json!(7))]));
    assert!(not_text.contains(&"Prop 'spare': until_bunk must be text (a bunk prop id).".to_string()), "{not_text:?}");
    let not_bunk = errors(json!([bunk(), held("spare", 3, 2, json!("bunk_9"))]));
    assert!(not_bunk.contains(&"Prop 'spare': until_bunk 'bunk_9' is not a bunk prop of this map (an id bunk_<n>).".to_string()), "{not_bunk:?}");
    let mut b = bunk();
    b["until_bunk"] = json!("bunk_7");
    assert_eq!(errors(json!([b])), ["Prop 'bunk_7': a bunk prop takes no until_bunk."]);
    assert_eq!(errors(json!([bunk(), held("spare", 3, 2, json!(""))])), ["Props 'bunk_7' and 'spare' stand on the same tile (3, 3)."], "no hold: the tiles are shared");
}

#[test]
fn a_prop_holding_one_bunk_does_not_share_anothers_tiles() {
    let other = json!({"id": "bunk_8", "art": "cot", "x": 3, "y": 3, "w": 1, "h": 1});
    let errors = map::prop_errors(&template(json!([other, held("spare", 3, 2, json!("bunk_7")), bunk()])));
    assert!(errors.iter().any(|e| e == "Props 'bunk_8' and 'spare' stand on the same tile (3, 3)."), "{errors:?}");
}

#[test]
fn the_writer_leaves_an_empty_until_bunk_out() {
    let mut t = template(json!([bunk(), held("spare", 6, 1, json!(""))]));
    source::map_without_reveal_defaults(&mut t);
    let props = t["props"].as_array().unwrap();
    assert!(props.iter().all(|p| p.get(source::PROP_UNTIL_BUNK).is_none()), "{props:?}");
}
