//! FB-206 (2b) and ASSETORG4 (FB-225): the overlay primitives' generic names (`lines`, `point_glow`, `particles_rise`,
//! `drift`) are the only names a sidecar may hold. There are no read-time aliases: a key outside `FACE_KEYS` is refused
//! by `parse_face`, an idle effect outside `IDLE_OVERLAYS` is a `face_problems` problem, and names are kept as written.

use aegis_core::assets::{face_problems, parse_face, FACE_KEYS, IDLE_OVERLAYS};
use serde_json::json;

#[test]
fn a_canonical_sidecar_parses_with_its_names_as_written() {
    let new = json!({"size": 96, "eyes": [[30, 40, 10, 6]], "glow": [[48, 70, 10]], "lines": [[50, 50, 90, 20]],
        "idle": [{"effect": "lines", "intensity": 0.7}, {"effect": "point_glow", "color": "#c084fc", "intensity": 0.5}, {"effect": "particles_rise", "intensity": 0.3}, {"effect": "drift", "intensity": 0.4}]});
    let a = parse_face(&new).unwrap();
    assert_eq!(a.lines, vec![[50, 50, 90, 20]]);
    assert_eq!(a.idle.iter().map(|o| o.effect.as_str()).collect::<Vec<_>>(), ["lines", "point_glow", "particles_rise", "drift"]);
    assert!(face_problems(&a, 96).is_empty(), "{:?}", face_problems(&a, 96));
}

#[test]
fn any_other_anchor_key_is_refused_without_an_alias() {
    // a key spelled any other way than `lines` (an old or misspelt name) is not read as it
    let other = json!({"size": 96, "roots": [[50, 50, 90, 20]]});
    assert_eq!(parse_face(&other).unwrap_err(), [format!("the face sidecar has no key 'roots' (keys: {}).", FACE_KEYS.join(", "))]);
    let bad_root = parse_face(&json!({"size": 96, "lines": [[50, 50, 400, 20]]})).unwrap();
    assert_eq!(face_problems(&bad_root, 96), ["line root 1 deg must be -180-360 (is 400)."]);
    assert_eq!(parse_face(&json!({"size": 96, "lines": [1]})).unwrap_err(), ["lines must be a list of [x, y, deg, len] lists of whole numbers."]);
}

#[test]
fn an_effect_outside_the_overlays_is_kept_as_written_and_flagged() {
    let no_root = parse_face(&json!({"size": 96, "idle": [{"effect": "lines"}, {"effect": "point_glow"}]})).unwrap();
    assert_eq!(face_problems(&no_root, 96), ["idle lines needs a line root.", "idle point_glow needs an eye box."]);
    let other = parse_face(&json!({"size": 96, "idle": [{"effect": "sparks", "intensity": 0.5}]})).unwrap();
    assert_eq!(other.idle[0].effect, "sparks", "no name is mapped to another");
    assert_eq!(face_problems(&other, 96), [format!("idle 'sparks' is not an overlay (one of {}).", IDLE_OVERLAYS.join(", "))]);
}
