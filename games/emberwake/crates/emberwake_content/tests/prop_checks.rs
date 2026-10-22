//! PROPS1 map props (games/emberwake/docs/specs/PROPS1.md sections 1-3): a prop's defaults and key order
//! (`source::prop_full`), the template checks P1-P12 (`check::map::prop_errors`, one failing case each), the art chain
//! (`art::prop_dirs`) and the asset checks A7-A10 (`check::assets::prop_warnings`) on a test-only map, world and
//! scratch asset layer. Expectations follow from the spec's rules and messages, never from shipped content.

use aegis_core::assets::Layers;
use aegis_core::json::{self, Dict};
use emberwake_content::check::{assets, map, View};
use emberwake_content::{art, source};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// 8 x 8: a wall border, a wall block in row 3 (columns 0-4), open floor elsewhere.
const ROWS: [&str; 8] = ["########", "#......#", "#......#", "#####..#", "#......#", "#......#", "#......#", "########"];

fn template(props: Value) -> Dict {
    let t = json!({"id": "test_props_map", "name": "Test Props", "rows": ROWS, "zones": [], "zone_legend": {},
        "anchors": [{"id": "spawn", "x": 5, "y": 5}, {"id": "well", "x": 1, "y": 2}], "paths": {"patrol": [[4, 2], [5, 2]]},
        "props": props, "tags": []});
    t.as_object().cloned().unwrap()
}

fn errors(props: Value) -> Vec<String> {
    map::prop_errors(&template(props))
}

#[test]
fn a_prop_gets_its_defaults_in_key_order() {
    let full = source::prop_full(json!({"y": 3, "x": 1, "art": "post", "id": "post"}).as_object().unwrap());
    assert_eq!(full.keys().collect::<Vec<_>>(), source::PROP_KEYS.iter().collect::<Vec<_>>());
    assert_eq!(Value::Object(full), json!({"id": "post", "art": "post", "x": 1, "y": 3, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false}));
    let odd = source::prop_full(json!({"id": "post", "colour": "red", "x": 2.0}).as_object().unwrap());
    assert_eq!(odd.keys().collect::<Vec<_>>(), ["id", "x", "w", "h", "layer", "variant", "flip", "colour"], "required keys stay missing, unknown ones follow");
    assert!(json::is_int(&odd["x"]), "whole numbers read as ints");
}

#[test]
fn valid_props_pass() {
    let props = json!([
        {"id": "hall", "art": "keep_hall", "x": 0, "y": 3, "w": 5, "h": 1},
        {"id": "post", "art": "signpost", "x": 7, "y": 4, "flip": true, "variant": 15},
        {"id": "rug", "art": "cobbles", "x": 1, "y": 1, "w": 6, "h": 6, "layer": "ground"},
        {"id": "rug_2", "art": "cobbles", "x": 0, "y": 0, "w": 8, "h": 8, "layer": "ground", "variant": -1}
    ]);
    assert_eq!(errors(props), Vec::<String>::new());
    assert!(errors(json!([])).is_empty());
    let mut no_props = template(json!([]));
    no_props.shift_remove("props");
    assert!(map::prop_errors(&no_props).is_empty());
}

#[test]
fn each_check_gives_exactly_its_message() {
    let one = |extra: Value| {
        let mut p = json!({"id": "post", "art": "post", "x": 0, "y": 0});
        for (k, v) in extra.as_object().unwrap() {
            p[k] = v.clone();
        }
        errors(json!([p]))
    };
    // P1
    assert_eq!(errors(json!("post")), ["Props must be a list of objects."]);
    assert_eq!(errors(json!([{"id": "post", "art": "post", "x": 0, "y": 0}, 3])), ["Props must be a list of objects."]);
    // P2
    let many: Vec<Value> = (0..=map::PROPS_MAX).map(|i| json!({"id": format!("p{i}"), "art": "rug", "x": 1, "y": 1, "layer": "ground"})).collect();
    assert_eq!(errors(Value::Array(many)), [format!("At most {} props per map (got {}).", map::PROPS_MAX, map::PROPS_MAX + 1)]);
    // P3
    assert_eq!(one(json!({"colour": "red"})), ["Prop 'post': unknown field 'colour'."]);
    // P4
    assert_eq!(one(json!({"id": "Post"})), ["Prop 0: id 'Post' must be lowercase [a-z0-9_], 2-24 characters."]);
    assert_eq!(one(json!({"id": "p"})), ["Prop 0: id 'p' must be lowercase [a-z0-9_], 2-24 characters."]);
    assert_eq!(errors(json!([{"art": "post", "x": 0, "y": 0}])), ["Prop 0: id '' must be lowercase [a-z0-9_], 2-24 characters."]);
    // P5
    let rug = json!({"id": "rug", "art": "rug", "x": 1, "y": 1, "layer": "ground"});
    assert_eq!(errors(json!([rug, rug])), ["Prop id 'rug' is used twice."]);
    // P6
    assert_eq!(one(json!({"art": "Big-Post"})), ["Prop 'post': art 'Big-Post' must be lowercase [a-z0-9_], 2-32 characters."]);
    assert_eq!(one(json!({"art": "a".repeat(33)})), [format!("Prop 'post': art '{}' must be lowercase [a-z0-9_], 2-32 characters.", "a".repeat(33))]);
    // P7
    let p7 = "Prop 'post': x and y are whole numbers; w and h are 1-16.";
    assert_eq!(one(json!({"x": 1.5})), [p7]);
    assert_eq!(one(json!({"w": 0})), [p7]);
    assert_eq!(one(json!({"h": 17})), [p7]);
    assert_eq!(errors(json!([{"id": "post", "art": "post", "x": 0}])), [p7], "y is required");
    // P8
    assert_eq!(one(json!({"x": 6, "w": 3})), ["Prop 'post': its footprint (6..8, 0..0) leaves the map (8 x 8)."]);
    assert_eq!(one(json!({"x": -1})), ["Prop 'post': its footprint (-1..-1, 0..0) leaves the map (8 x 8)."]);
    // P9: the first tile that is not a wall, row by row
    assert_eq!(one(json!({"w": 2, "h": 2})), ["Prop 'post': a standing prop needs wall tiles under its whole footprint; (1, 1) is '.'."]);
    assert!(one(json!({"w": 2, "h": 2, "layer": "ground"})).is_empty(), "a ground prop lies on any tile");
    // P10
    let pair = json!([{"id": "wall_a", "art": "rubble", "x": 0, "y": 3, "w": 3}, {"id": "wall_b", "art": "rubble", "x": 2, "y": 3, "w": 3}]);
    assert_eq!(errors(pair), ["Props 'wall_a' and 'wall_b' stand on the same tile (2, 3)."]);
    // P11
    assert_eq!(one(json!({"layer": "over"})), ["Prop 'post': layer must be sort or ground."]);
    // P12
    let p12 = "Prop 'post': variant is -1 (by position) or 0-15; flip is true or false.";
    assert_eq!(one(json!({"variant": 16})), [p12]);
    assert_eq!(one(json!({"variant": -2})), [p12]);
    assert_eq!(one(json!({"flip": "yes"})), [p12]);
}

#[test]
fn prop_art_follows_the_world_lineage_then_common() {
    assert_eq!(art::prop_dirs(&["ashlands".into()], "tent"), ["props/ashlands/tent", "props/common/tent"]);
    assert_eq!(art::prop_dirs(&["tidewrack".into(), "ashlands".into()], "tent"), ["props/tidewrack/tent", "props/ashlands/tent", "props/common/tent"]);
}

/// A file whose header says it is a `w` x `h` PNG (the checks read only the header).
fn png(path: &Path, w: u32, h: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut b = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    std::fs::write(path, b).unwrap();
}

/// The world's `post` art (two files, `post_h` px tall), a 32 x 32 `crate` and a 64 x 64 `rug` in common, no `lamp`.
fn layer(name: &str, post_h: u32) -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_prop_checks_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for v in 0..2 {
        png(&root.join(format!("props/test_world/post/v{v}.png")), 32, post_h);
    }
    png(&root.join("props/common/crate/v0.png"), 32, 32);
    png(&root.join("props/common/rug/v0.png"), 64, 64);
    root
}

fn view(root: &Path) -> View {
    let props = json!([
        {"id": "post", "art": "post", "x": 1, "y": 3, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false},
        {"id": "lamp_a", "art": "lamp", "x": 0, "y": 0, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false},
        {"id": "lamp_b", "art": "lamp", "x": 7, "y": 0, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": false},
        {"id": "crates", "art": "crate", "x": 0, "y": 7, "w": 2, "h": 1, "layer": "sort", "variant": -1, "flip": false},
        {"id": "rug", "art": "rug", "x": 5, "y": 5, "w": 1, "h": 1, "layer": "ground", "variant": -1, "flip": false},
        {"id": "post_2", "art": "post", "x": 7, "y": 4, "w": 1, "h": 1, "layer": "sort", "variant": 5, "flip": false},
        {"id": "column", "art": "post", "x": 4, "y": 3, "w": 1, "h": 1, "layer": "sort", "variant": -1, "flip": true}
    ]);
    let worlds = json!({"test_world": {"manifest": {"id": "test_world"}, "own": {"locations": [
        {"id": "spot", "map": "test_props_map"}, {"id": "spot_2", "map": "test_props_map"}]}}});
    let game = json!({"maps": [Value::Object(template(props))]});
    View::new(&json!({}), &worlds, &game, Layers::of(vec![root.to_path_buf()]))
}

#[test]
fn asset_checks_name_missing_art_sizes_variants_and_hidden_tiles() {
    let a7 = "Map 'test_props_map' prop art 'lamp': no file props/{test_world,common}/lamp/v0.png for world 'test_world'; it draws its shape.";
    let a8_sort = "Prop art 'props/common/crate/v0.png' is 32x32 px; a 2 x 1 sort prop wants 64-96 x 32-160 px.";
    let a8_ground = "Prop art 'props/common/rug/v0.png' is 64x64 px; a 1 x 1 ground prop wants 32 x 32 px.";
    let a9 = "Map 'test_props_map' prop 'post_2': variant 5, but 'post' has 2 files; it shows v0.";
    // 32 x 80 art on a 1 x 1 standing prop reaches 1.5 tiles above its footprint: the anchor north of `post` and the
    // patrol tile north of `column` lie under it.
    let tall = layer("tall", 80);
    let found = assets::prop_warnings(&view(&tall), "test_world");
    let a10_anchor = "Map 'test_props_map' prop 'post' hides 'well' at (1, 2) behind its art.";
    let a10_path = "Map 'test_props_map' prop 'column' hides 'patrol' at (4, 2) behind its art.";
    assert_eq!(found, [a10_anchor, a7, a8_sort, a8_ground, a9, a10_path], "each line once, in prop order");
    // 32 x 32 art stays on its footprint
    let short = layer("short", 32);
    assert_eq!(assets::prop_warnings(&view(&short), "test_world"), [a7, a8_sort, a8_ground, a9]);
    let mut no_layers = view(&short);
    no_layers.assets = Layers::default();
    assert!(assets::prop_warnings(&no_layers, "test_world").is_empty(), "skipped without asset layers");
    let _ = std::fs::remove_dir_all(&tall);
    let _ = std::fs::remove_dir_all(&short);
}
