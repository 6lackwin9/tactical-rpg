//! NPCFACE1: a person's marker takes an optional `face` (`down`, the default, `up`, `left`, `right`). A bad value or a
//! `face` on another kind is a check problem; the default is never stored. Scratch rows only.

use aegis_core::assets::Layers;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{map, View};
use rpgkit_content::store;
use serde_json::{json, Value};

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

/// The problems of one marker on a small map.
fn problems(marker: Value) -> Vec<String> {
    let t = json!({"id": "m", "rows": ["..", ".."], "anchors": [{"id": "a", "x": 0, "y": 0}]});
    let location = dict(json!({"id": "here", "map": "m", "markers": [marker]}));
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {"locations": [location], "characters": [{"id": "who", "name": "Who"}]}}});
    let view = View::new(&json!({}), &worlds, &json!({"maps": [t]}), Layers::default());
    map::location_errors(&view, "w", &location)
}

fn npc(face: Option<Value>) -> Value {
    let mut m = json!({"id": "n", "kind": "npc", "anchor": "a", "character": "who"});
    if let Some(f) = face {
        m["face"] = f;
    }
    m
}

const BAD: &str = "Marker 'n': face must be one of down, up, left, right.";

#[test]
fn every_named_face_and_no_face_pass() {
    assert_eq!(problems(npc(None)), Vec::<String>::new());
    for f in ["down", "up", "left", "right"] {
        assert_eq!(problems(npc(Some(json!(f)))), Vec::<String>::new(), "{f}");
    }
}

#[test]
fn a_bad_face_is_a_problem() {
    for bad in [json!("sideways"), json!(""), json!(3), Value::Null] {
        assert_eq!(problems(npc(Some(bad.clone()))), [BAD], "{bad}");
    }
}

#[test]
fn only_people_take_a_face() {
    let m = json!({"id": "n", "kind": "point", "anchor": "a", "face": "up"});
    assert_eq!(problems(m), ["Marker 'n' (point): unknown key 'face'."]);
}

#[test]
fn the_default_face_is_never_stored() {
    let stored = |face: Option<Value>| {
        let row = dict(json!({"id": "here", "name": "Here", "markers": [npc(face)]}));
        json::dict(&store::world_storage("locations", &row)["markers"][0]).get("face").cloned()
    };
    assert_eq!(stored(None), None);
    assert_eq!(stored(Some(json!("down"))), None);
    assert_eq!(stored(Some(json!("up"))), Some(json!("up")));
    // a bad value stays for the check to report
    assert_eq!(stored(Some(json!("sideways"))), Some(json!("sideways")));
}
