//! FB-206 (2b): the overlay primitives' generic names (`lines`, `point_glow`, `particles_rise`, `drift`) and the old
//! names a sidecar may still hold (`veins`, `eye_glow`, `ash`, `smoke`; the anchor key `veins`). A sidecar written
//! with the old names parses to the same face as one with the new names; both names of one key are refused; the
//! problems name the canonical names.

use aegis_core::assets::{canonical_primitive, face_problems, parse_face, FACE_KEYS, FACE_KEY_ALIASES, IDLE_OVERLAYS, PRIMITIVE_ALIASES};
use serde_json::json;

#[test]
fn every_old_name_maps_to_a_canonical_overlay_and_other_names_stay() {
    for (old, new) in PRIMITIVE_ALIASES {
        assert_eq!(canonical_primitive(old), new);
        assert!(IDLE_OVERLAYS.contains(&new), "{new} is an idle overlay");
        assert!(!IDLE_OVERLAYS.contains(&old), "{old} is no longer listed");
        assert_eq!(canonical_primitive(new), new, "a canonical name maps to itself");
    }
    for name in ["glow", "flash", "face", "sparkle", ""] {
        assert_eq!(canonical_primitive(name), name);
    }
    for (old, new) in FACE_KEY_ALIASES {
        assert!(FACE_KEYS.contains(&new) && !FACE_KEYS.contains(&old), "{old} -> {new}");
    }
}

#[test]
fn an_old_sidecar_parses_to_the_same_face_as_a_new_one() {
    let old = json!({"size": 96, "eyes": [[30, 40, 10, 6]], "glow": [[48, 70, 10]], "veins": [[50, 50, 90, 20]],
        "idle": [{"effect": "veins", "intensity": 0.7}, {"effect": "eye_glow", "color": "#c084fc", "intensity": 0.5}, {"effect": "ash", "intensity": 0.3}, {"effect": "smoke", "intensity": 0.4}]});
    let new = json!({"size": 96, "eyes": [[30, 40, 10, 6]], "glow": [[48, 70, 10]], "lines": [[50, 50, 90, 20]],
        "idle": [{"effect": "lines", "intensity": 0.7}, {"effect": "point_glow", "color": "#c084fc", "intensity": 0.5}, {"effect": "particles_rise", "intensity": 0.3}, {"effect": "drift", "intensity": 0.4}]});
    let (a, b) = (parse_face(&old).unwrap(), parse_face(&new).unwrap());
    assert_eq!(a, b, "the old names load as the new ones, in the sidecar's order");
    assert_eq!(a.lines, vec![[50, 50, 90, 20]]);
    assert_eq!(a.idle.iter().map(|o| o.effect.as_str()).collect::<Vec<_>>(), ["lines", "point_glow", "particles_rise", "drift"]);
    assert!(face_problems(&a, 96).is_empty(), "{:?}", face_problems(&a, 96));
}

#[test]
fn both_names_of_one_key_are_refused_and_problems_name_the_canonical_names() {
    let both = json!({"size": 96, "lines": [[50, 50, 90, 20]], "veins": [[50, 50, 90, 20]]});
    assert_eq!(parse_face(&both).unwrap_err(), ["the face sidecar holds both 'lines' and its old name 'veins' (keep 'lines')."]);
    let no_root = parse_face(&json!({"size": 96, "idle": [{"effect": "veins"}, {"effect": "eye_glow"}]})).unwrap();
    assert_eq!(face_problems(&no_root, 96), ["idle lines needs a line root.", "idle point_glow needs an eye box."]);
    let bad_root = parse_face(&json!({"size": 96, "veins": [[50, 50, 400, 20]]})).unwrap();
    assert_eq!(face_problems(&bad_root, 96), ["line root 1 deg must be -180-360 (is 400)."]);
    assert_eq!(parse_face(&json!({"size": 96, "veins": [1]})).unwrap_err(), ["lines must be a list of [x, y, deg, len] lists of whole numbers."]);
}
