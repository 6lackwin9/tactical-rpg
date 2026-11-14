//! PR5-48 (MAP-LAYOUT-ED), clause 2: the world map layout sidecar check and writer (`world_map`) on scratch worlds and
//! asset layers. A sidecar named for a world or a region checks its landmark ids (locations of that world) and route ids
//! (routes of that world), every position and route point (two finite numbers in `POS_RANGE`) and scale
//! (`SCALE_RANGE`), and warns for a landmark without its image (naming the kind pin the map draws); a pre-staged name
//! (a background, no data) skips the id checks; any other name is ignored. The writer applies layout edits, checks, and
//! writes the canonical text into the first layer, or refuses with nothing written.

use aegis_core::assets::Layers;
use aegis_core::scratch::Scratch;
use rpgkit_content::check::{self, View};
use rpgkit_content::world_map::{self, Owner, POS_RANGE, SCALE_RANGE};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn view(layers: Layers) -> View {
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {
        "regions": [{"id": "quarter", "name": "Quarter"}],
        "locations": [{"id": "hub", "name": "Hub", "kind": "town", "region": "quarter"}, {"id": "gate", "name": "Gate", "kind": "gate", "region": "quarter"}, {"id": "wilds", "name": "Wilds", "region": "quarter"}],
        "routes": [{"id": "road", "from": "hub", "to": "gate"}],
    }}});
    View::new(&json!({}), &worlds, &json!({}), layers)
}

/// A scratch asset layer (its root `assets/`), with the files given (relative path, text).
fn layer(s: &Scratch, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = s.path().join(name).join("assets");
    for (rel, text) in files {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

fn errors(v: &View, name: &str, layout: Value) -> Vec<String> {
    world_map::check(&v.worlds, &v.assets, name, &layout).errors
}

const CLEAN: &str = r#"{"landmarks": {"hub": {"pos": [0.5, 1.042], "scale": 1.0}, "gate": {"pos": [-0.01, 0.2], "scale": 1.5}}, "routes": {"road": [[0.4, 0.8], [0.3, 0.5]]}}"#;

#[test]
fn a_names_owner_is_a_region_a_world_a_prestaged_background_or_nothing() {
    let s = Scratch::new("wml_owner");
    let root = layer(&s, "game", &[("backgrounds/world_map/high.png", "png")]);
    let v = view(Layers::of(vec![root]));
    assert_eq!(world_map::owner(&v.worlds, &v.assets, "quarter"), Owner::Region { world: "w".into() });
    assert_eq!(world_map::owner(&v.worlds, &v.assets, "w"), Owner::World("w".into()));
    assert_eq!(world_map::owner(&v.worlds, &v.assets, "high"), Owner::PreStaged);
    assert_eq!(world_map::owner(&v.worlds, &v.assets, "stray"), Owner::Unknown);
    assert_eq!(Owner::PreStaged.world(), None);
}

#[test]
fn a_clean_sidecar_has_no_errors_and_warns_for_each_missing_landmark_image() {
    let s = Scratch::new("wml_clean");
    let root = layer(&s, "game", &[("backgrounds/world_map/quarter.json", CLEAN)]);
    let v = view(Layers::of(vec![root.clone()]));
    let f = world_map::check(&v.worlds, &v.assets, "quarter", &serde_json::from_str(CLEAN).unwrap());
    assert!(f.errors.is_empty(), "{:?}", f.errors);
    assert_eq!(f.warnings, vec![
        "Landmark 'hub' has no map/landmarks/hub.png: the map draws its pin ui/pins/town.png.".to_string(),
        "Landmark 'gate' has no map/landmarks/gate.png: the map draws its pin ui/pins/gate.png.".to_string(),
    ]);
    // the image there: no warning for it; a location without a kind names its kind pin
    std::fs::create_dir_all(root.join("map/landmarks")).unwrap();
    std::fs::write(root.join("map/landmarks/hub.png"), "png").unwrap();
    let f = world_map::check(&v.worlds, &v.assets, "quarter", &json!({"landmarks": {"hub": {"pos": [0.5, 0.5]}, "wilds": {"pos": [0.1, 0.1]}}}));
    assert_eq!(f.warnings, vec!["Landmark 'wilds' has no map/landmarks/wilds.png: the map draws its kind pin.".to_string()]);
    // the same sidecar named for the world checks the same ids
    assert!(errors(&v, "w", serde_json::from_str(CLEAN).unwrap()).is_empty());
}

#[test]
fn unknown_landmark_and_route_ids_are_errors() {
    let v = view(Layers::default());
    let found = errors(&v, "quarter", json!({"landmarks": {"keep": {"pos": [0.5, 0.5]}}, "routes": {"lane": [[0.1, 0.1]]}}));
    assert_eq!(found, vec!["Landmark 'keep' isn't a location of world 'w'.".to_string(), "Route 'lane' isn't a route of world 'w'.".to_string()]);
    // with no asset layer there is nothing to warn about
    assert!(world_map::check(&v.worlds, &v.assets, "quarter", &json!({"landmarks": {"hub": {"pos": [0.5, 0.5]}}})).warnings.is_empty());
}

#[test]
fn positions_and_route_points_must_be_two_finite_numbers_in_range() {
    let v = view(Layers::default());
    let (lo, hi) = POS_RANGE;
    assert!(errors(&v, "quarter", json!({"landmarks": {"hub": {"pos": [lo, hi]}}, "routes": {"road": [[hi, lo]]}})).is_empty(), "the range's ends");
    for bad in [json!([hi + 0.01, 0.5]), json!([0.5, lo - 0.01]), json!([0.5]), json!([0.5, 0.5, 0.5]), json!(["a", 0.5]), json!(null), json!({"x": 0.5})] {
        let found = errors(&v, "quarter", json!({"landmarks": {"hub": {"pos": bad.clone()}}}));
        assert_eq!(found, vec![format!("Landmark 'hub': pos must be [x, y], two numbers in -0.25..=1.25 (got {bad}).")], "{bad}");
        let found = errors(&v, "quarter", json!({"routes": {"road": [[0.5, 0.5], bad.clone()]}}));
        assert_eq!(found, vec![format!("Route 'road' point 1: must be [x, y], two numbers in -0.25..=1.25 (got {bad}).")], "{bad}");
    }
    assert_eq!(errors(&v, "quarter", json!({"landmarks": {"hub": {"scale": 1.0}}})), vec!["Landmark 'hub' has no pos.".to_string()]);
    assert_eq!(errors(&v, "quarter", json!({"routes": {"road": {"a": 1}}})), vec!["Route 'road' is a list of [x, y] points.".to_string()]);
    // JSON holds no infinity: a file that writes one is not JSON (the check says so)
    let s = Scratch::new("wml_inf");
    let root = layer(&s, "game", &[("backgrounds/world_map/quarter.json", r#"{"landmarks": {"hub": {"pos": [1e999, 0.5]}}}"#)]);
    let all = world_map::check_all(&view(Layers::of(vec![root])).worlds, &Layers::of(vec![s.path().join("game/assets")]));
    assert_eq!(all.len(), 1);
    assert!(all[0].1.errors[0].starts_with("Not JSON:"), "{:?}", all[0].1.errors);
}

#[test]
fn a_scale_must_be_a_number_in_range() {
    let v = view(Layers::default());
    let (lo, hi) = SCALE_RANGE;
    assert!(errors(&v, "quarter", json!({"landmarks": {"hub": {"pos": [0.5, 0.5], "scale": lo}, "gate": {"pos": [0.5, 0.5], "scale": hi}}})).is_empty());
    for bad in [json!(lo - 0.01), json!(hi + 0.01), json!(0), json!(-1.0), json!("big")] {
        let found = errors(&v, "quarter", json!({"landmarks": {"hub": {"pos": [0.5, 0.5], "scale": bad.clone()}}}));
        assert_eq!(found, vec![format!("Landmark 'hub': scale must be a number in 0.25..=4.0 (got {bad}).")], "{bad}");
    }
}

#[test]
fn the_shape_is_landmarks_and_routes_only() {
    let v = view(Layers::default());
    assert_eq!(errors(&v, "quarter", json!([1, 2])), vec!["A layout is an object {landmarks, routes}.".to_string()]);
    assert_eq!(errors(&v, "quarter", json!({"pins": {}})), vec!["Unknown key 'pins' (landmarks, routes).".to_string()]);
    assert_eq!(errors(&v, "quarter", json!({"landmarks": {"hub": {"pos": [0.5, 0.5], "tint": 1}}})), vec!["Landmark 'hub': unknown key 'tint' (pos, scale).".to_string()]);
    assert_eq!(errors(&v, "quarter", json!({"landmarks": [], "routes": 3})), vec!["'landmarks' is an object {location id: {pos, scale}}.".to_string(), "'routes' is an object {route id: [[x, y], ...]}.".to_string()]);
    assert!(errors(&v, "quarter", json!({})).is_empty(), "an empty layout");
}

#[test]
fn a_prestaged_name_skips_the_id_checks_and_other_names_are_ignored() {
    let s = Scratch::new("wml_prestaged");
    let staged = r#"{"landmarks": {"spire_gate": {"pos": [0.5, 0.158], "scale": 1.35}}, "routes": {"climb": [[0.5, 0.3]]}}"#;
    let root = layer(&s, "game", &[
        ("backgrounds/world_map/high.png", "png"),
        ("backgrounds/world_map/high.json", staged),
        ("backgrounds/world_map/stray.json", r#"{"landmarks": {"nowhere": {"pos": [9, 9]}}}"#),
        ("backgrounds/world_map/Bad-Name.json", "not json"),
        ("backgrounds/world_map/notes.txt", "x"),
    ]);
    let v = view(Layers::of(vec![root]));
    let f = world_map::check(&v.worlds, &v.assets, "high", &serde_json::from_str(staged).unwrap());
    assert_eq!(f, world_map::Findings::default(), "no id errors and no image warnings");
    // still its numbers
    assert_eq!(errors(&v, "high", json!({"landmarks": {"spire_gate": {"pos": [0.5, 0.5], "scale": 9}}})).len(), 1);
    let all: Vec<String> = world_map::check_all(&v.worlds, &v.assets).into_iter().map(|(n, _)| n).collect();
    assert_eq!(all, vec!["high".to_string()], "stray (no data, no background) and the non-id files are ignored");
    assert_eq!(world_map::sidecar_names(&v.assets), vec!["high".to_string(), "stray".to_string()]);
    assert_eq!(world_map::background_names(&v.assets), vec!["high".to_string()]);
}

#[test]
fn the_content_check_reports_sidecar_errors_and_warnings_by_name() {
    let s = Scratch::new("wml_report");
    let bad = r#"{"landmarks": {"keep": {"pos": [0.5, 0.5]}, "hub": {"pos": [0.5, 0.5]}}}"#;
    let root = layer(&s, "game", &[("backgrounds/world_map/quarter.json", bad)]);
    let v = view(Layers::of(vec![root]));
    assert_eq!(check::validate_all(&v).get("world_map:quarter").cloned().unwrap_or_default(), vec!["Landmark 'keep' isn't a location of world 'w'.".to_string()]);
    assert_eq!(check::warnings_all(&v).get("world_map:quarter").map(Vec::len), Some(1), "the hub's image");
    // no asset layer: no sidecar is looked at
    let none = view(Layers::default());
    assert!(!check::validate_all(&none).keys().any(|k| k.starts_with("world_map:")));
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn the_canonical_text_keeps_the_order_and_the_editors_precision() {
    let v: Value = serde_json::from_str(r#"{"routes": {"b": [[0.123456, -0.00001]], "a": []}, "landmarks": {"z": {"pos": [0.6, 0.99]}, "y": {"pos": [1, 0], "scale": 1.23456}}}"#).unwrap();
    let want = "{\n  \"landmarks\": {\n    \"z\": {\"pos\": [0.6, 0.99], \"scale\": 1.0},\n    \"y\": {\"pos\": [1.0, 0.0], \"scale\": 1.235}\n  },\n  \"routes\": {\n    \"b\": [[0.1235, 0.0]],\n    \"a\": []\n  }\n}\n";
    assert_eq!(world_map::text(&v), want);
    assert_eq!(world_map::text(&json!({})), "{\n  \"landmarks\": {},\n  \"routes\": {}\n}\n");
    // the canonical text is its own canonical text
    assert_eq!(world_map::text(&serde_json::from_str(want).unwrap()), want);
}

#[test]
fn the_writer_applies_edits_checks_and_writes_the_canonical_text() {
    let s = Scratch::new("wml_write");
    let canonical = world_map::text(&serde_json::from_str(CLEAN).unwrap());
    let root = layer(&s, "game", &[("backgrounds/world_map/quarter.json", &canonical)]);
    let v = view(Layers::of(vec![root.clone()]));
    let path = root.join("backgrounds/world_map/quarter.json");
    // no edits: the canonical file is written again byte for byte
    let w = world_map::write(&v.worlds, &v.assets, "quarter", &json!([])).expect("a re-save");
    assert_eq!((w.path.clone(), w.edits, w.changed), (path.clone(), 0, false));
    assert_eq!(read(&path), canonical);
    // the layout editor's edits (LayoutEdit::to_json): a moved and sized landmark, a new one, a route's points
    let edits = json!([
        {"id": "hub", "pos": [0.25, 0.75], "scale": 1.25},
        {"id": "wilds", "pos": [0.1, 0.2]},
        {"route": "road", "points": [[0.3, 0.3], [0.35, 0.4], [0.4, 0.5]]},
    ]);
    let w = world_map::write(&v.worlds, &v.assets, "quarter", &edits).expect("valid edits");
    assert_eq!((w.edits, w.changed), (3, true));
    let back: Value = serde_json::from_str(&read(&path)).unwrap();
    assert_eq!(back["landmarks"]["hub"], json!({"pos": [0.25, 0.75], "scale": 1.25}));
    assert_eq!(back["landmarks"]["wilds"], json!({"pos": [0.1, 0.2], "scale": 1.0}), "a new landmark at scale 1");
    assert_eq!(back["routes"]["road"], json!([[0.3, 0.3], [0.35, 0.4], [0.4, 0.5]]));
    let keys: Vec<&String> = back["landmarks"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["hub", "gate", "wilds"], "the file's order, a new entry last");
    assert_eq!(read(&path), world_map::text(&back), "canonical");
    // one edit object alone, the scale only
    world_map::write(&v.worlds, &v.assets, "quarter", &json!({"id": "gate", "scale": 2})).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&read(&path)).unwrap()["landmarks"]["gate"], json!({"pos": [-0.01, 0.2], "scale": 2.0}));
}

#[test]
fn the_writer_refuses_anything_invalid_with_nothing_written() {
    let s = Scratch::new("wml_refuse");
    let root = layer(&s, "game", &[("backgrounds/world_map/quarter.json", CLEAN)]);
    let v = view(Layers::of(vec![root.clone()]));
    let path = root.join("backgrounds/world_map/quarter.json");
    let refused = |edits: Value, name: &str| world_map::write(&v.worlds, &v.assets, name, &edits).expect_err("refused");
    let cases: Vec<(Value, &str, &str)> = vec![
        (json!([{"id": "keep", "pos": [0.5, 0.5]}]), "quarter", "isn't a location of world 'w'"),
        (json!([{"route": "lane", "points": []}]), "quarter", "isn't a route of world 'w'"),
        (json!([{"id": "hub", "pos": [2.0, 0.5]}]), "quarter", "pos must be [x, y]"),
        (json!([{"id": "hub", "scale": 9}]), "quarter", "scale must be a number in 0.25..=4.0"),
        (json!([{"route": "road", "points": [[0.5, "x"]]}]), "quarter", "point 0"),
        (json!([{"id": "wilds", "scale": 2}]), "quarter", "needs a pos"),
        (json!([{"id": "hub"}]), "quarter", "needs pos and / or scale"),
        (json!([{"id": "hub", "route": "road"}]), "quarter", "unknown key 'id'"),
        (json!([{"pos": [0.1, 0.1]}]), "quarter", "needs id (a landmark) or route"),
        (json!("hub"), "quarter", "The edits are a list"),
        (json!([]), "stray", "nothing to lay out"),
        (json!([]), "Bad-Name", "isn't a sidecar name"),
    ];
    for (edits, name, says) in cases {
        let e = refused(edits.clone(), name);
        assert!(e.iter().any(|m| m.contains(says)), "{edits} on {name}: {e:?}");
        assert_eq!(read(&path), CLEAN, "{edits}: nothing written");
    }
    assert!(!root.join("backgrounds/world_map/stray.json").exists());
    // a file that is not JSON is refused too, and kept as it is
    std::fs::write(&path, "{ broken").unwrap();
    assert!(refused(json!([]), "quarter")[0].contains("not JSON"));
    assert_eq!(read(&path), "{ broken");
}

#[test]
fn the_writer_starts_a_prestaged_layout_and_writes_into_the_first_layer() {
    let s = Scratch::new("wml_layers");
    let game = layer(&s, "game", &[("backgrounds/world_map/high.png", "png")]);
    let kit = layer(&s, "kit", &[("backgrounds/world_map/quarter.json", CLEAN)]);
    let v = view(Layers::of(vec![game.clone(), kit.clone()]));
    // a pre-staged act: no file yet, any ids (its data has not landed), numbers checked
    let w = world_map::write(&v.worlds, &v.assets, "high", &json!([{"id": "spire_gate", "pos": [0.498, 0.158], "scale": 1.35}])).unwrap();
    assert_eq!(w.path, game.join("backgrounds/world_map/high.json"));
    assert_eq!(read(&w.path), "{\n  \"landmarks\": {\n    \"spire_gate\": {\"pos\": [0.498, 0.158], \"scale\": 1.35}\n  },\n  \"routes\": {}\n}\n");
    // a layout the kit layer holds: edited from it, written into the game's layer, the kit's file kept
    let w = world_map::write(&v.worlds, &v.assets, "quarter", &json!([{"id": "hub", "pos": [0.5, 0.9]}])).unwrap();
    assert_eq!(w.path, game.join("backgrounds/world_map/quarter.json"));
    assert_eq!(read(&kit.join("backgrounds/world_map/quarter.json")), CLEAN);
    let back: Value = serde_json::from_str(&read(&w.path)).unwrap();
    assert_eq!(back["landmarks"]["gate"]["scale"], json!(1.5), "the rest from the kit's file");
}
