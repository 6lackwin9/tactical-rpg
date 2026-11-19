//! PR7-37 (PT-03, PT-29): a point of interest takes an optional `use` (`rpgkit_data::POINT_USES`: "", rest, crypts,
//! stash): what acting on it does. Another value, or a `use` on another kind, is a check problem; the writer keeps the
//! row as written. Scratch rows only.

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
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {"locations": [location]}}});
    let view = View::new(&json!({}), &worlds, &json!({"maps": [t]}), Layers::default());
    map::location_errors(&view, "w", &location)
}

fn point(used: Option<Value>) -> Value {
    let mut m = json!({"id": "p", "kind": "point", "anchor": "a", "label": "The brazier", "note": "Warm."});
    if let Some(u) = used {
        m["use"] = u;
    }
    m
}

#[test]
fn every_named_use_and_no_use_pass() {
    assert_eq!(problems(point(None)), Vec::<String>::new());
    for u in rpgkit_data::POINT_USES {
        assert_eq!(problems(point(Some(json!(u)))), Vec::<String>::new(), "{u}");
    }
}

#[test]
fn a_bad_use_is_a_problem() {
    let bad_line = format!("Marker 'p': use must be one of {} (or \"\").", rpgkit_data::POINT_USES[1..].join(", "));
    for bad in [json!("dance"), json!(3), Value::Null] {
        assert_eq!(problems(point(Some(bad.clone()))), [bad_line.clone()], "{bad}");
    }
}

#[test]
fn only_points_take_a_use() {
    for kind in ["chest", "lore", "npc"] {
        let p = problems(json!({"id": "c", "kind": kind, "anchor": "a", "use": "rest"}));
        assert!(p.contains(&format!("Marker 'c' ({kind}): unknown key 'use'.")), "{kind}: {p:?}");
    }
}

#[test]
fn the_writer_keeps_a_points_use() {
    let stored = |used: Option<Value>| {
        let row = dict(json!({"id": "here", "name": "Here", "markers": [point(used)]}));
        json::dict(&store::world_storage("locations", &row)["markers"][0]).get("use").cloned()
    };
    assert_eq!(stored(None), None, "no use: none stored");
    assert_eq!(stored(Some(json!("rest"))), Some(json!("rest")));
}
