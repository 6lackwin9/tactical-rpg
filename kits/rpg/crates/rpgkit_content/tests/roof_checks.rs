//! REVEAL1 (docs/specs/REVEAL1.md, PR5-54): enterable indoor rooms in the map template. A zone legend entry's `indoor`
//! and `wall_set`, a prop's `roof` layer and `roof_of`, a map's own `boundary` and a ground row's `indoor`: the checks
//! (R1 refuses a roof without an indoor zone, R2 warns about an indoor cell no roof covers), the writer (every new key
//! left out at its default, so a map without them re-saves byte for byte) and the boundary's asset rules. Scratch
//! folders and test-only maps; the expectations follow from the spec's rules.

use aegis_core::assets::Layers;
use aegis_core::json::Dict;
use rpgkit_content::check::{self, assets, map, View};
use rpgkit_content::{art, source, store};
use rpgkit_data::{Catalog, GROUNDS_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// 12 x 8: walls round the edge; a 5 x 4 building (a wall ring at x 5-9, y 2-5) with its door at (7, 5).
const ROWS: [&str; 8] = ["############", "#..........#", "#....#####.#", "#....#...#.#", "#....#...#.#", "#....##.##.#", "#..........#", "############"];
/// The building's floor and door are zone `h` (the den); the yard outside is `y`.
const ZONES: [&str; 8] = ["yyyyyyyyyyyy", "yyyyyyyyyyyy", "yyyyyyyyyyyy", "yyyyyyhhhyyy", "yyyyyyhhhyyy", "yyyyyyyhyyyy", "yyyyyyyyyyyy", "yyyyyyyyyyyy"];

fn legend(indoor: Value) -> Value {
    json!({"y": {"id": "yard", "kind": "safe"}, "h": {"id": "den", "kind": "safe", "indoor": indoor, "wall_set": "zz_plaster"}})
}

fn roof(x: i64, y: i64, w: i64, h: i64, zone: &str) -> Value {
    json!({"id": "den_roof", "art": "den_house", "x": x, "y": y, "w": w, "h": h, "layer": "roof", "roof_of": zone})
}

fn template(legend: Value, props: Value) -> Dict {
    dict(json!({"id": "test_map", "name": "T", "rows": ROWS, "zones": ZONES, "zone_legend": legend, "anchors": [], "paths": {}, "props": props, "tags": []}))
}

/// The template as the loader gives it (props with their defaults).
fn loaded(t: &Dict) -> Dict {
    let mut t = t.clone();
    if let Some(Value::Array(list)) = t.get("props").cloned() {
        t.insert("props".into(), Value::Array(list.iter().map(|p| Value::Object(source::prop_full(p.as_object().unwrap()))).collect()));
    }
    t
}

fn errors_of(t: &Dict) -> Vec<String> {
    let t = loaded(t);
    let mut out = map::prop_errors(&t);
    out.extend(map::reveal_errors(&t));
    out
}

// ---- R1 ------------------------------------------------------------------------------------------------------------

#[test]
fn r1_a_roof_names_an_indoor_zone_and_only_a_roof_takes_roof_of() {
    // the whole building under its roof: no error, wall ring and open floor alike (no wall rule for a roof)
    assert_eq!(errors_of(&template(legend(json!(true)), json!([roof(5, 2, 5, 4, "den")]))), Vec::<String>::new());
    let one = |legend: Value, prop: Value| errors_of(&template(legend, json!([prop])));
    assert_eq!(one(legend(json!(true)), roof(5, 2, 5, 4, "")), ["Prop 'den_roof': a roof prop names its indoor zone in roof_of."]);
    let mut bare = roof(5, 2, 5, 4, "");
    bare.as_object_mut().unwrap().remove("roof_of");
    assert_eq!(one(legend(json!(true)), bare), ["Prop 'den_roof': a roof prop names its indoor zone in roof_of."]);
    let not_indoor = "Prop 'den_roof': roof_of 'den' is not an indoor zone of this map (a zone_legend entry with indoor: true).";
    assert_eq!(one(legend(json!(false)), roof(5, 2, 5, 4, "den")), [not_indoor]);
    assert_eq!(one(legend(json!(true)), roof(5, 2, 5, 4, "nowhere")), ["Prop 'den_roof': roof_of 'nowhere' is not an indoor zone of this map (a zone_legend entry with indoor: true)."]);
    assert_eq!(one(legend(json!(true)), roof(5, 2, 5, 4, "yard")), ["Prop 'den_roof': roof_of 'yard' is not an indoor zone of this map (a zone_legend entry with indoor: true)."]);
    assert_eq!(one(legend(json!(true)), json!({"id": "den_roof", "art": "den_house", "x": 5, "y": 2, "w": 5, "h": 4, "layer": "roof", "roof_of": 3})), ["Prop 'den_roof': roof_of must be text (a zone id)."]);
    // another layer: roof_of "" is its default, any other value is refused
    let mut on_wall = json!({"id": "sign", "art": "signpost", "x": 5, "y": 2, "roof_of": "den"});
    assert_eq!(one(legend(json!(true)), on_wall.clone()), ["Prop 'sign': only a roof prop takes roof_of."]);
    on_wall["roof_of"] = json!("");
    assert_eq!(one(legend(json!(true)), on_wall), Vec::<String>::new());
    assert_eq!(one(legend(json!(true)), json!({"id": "rug", "art": "rug", "x": 7, "y": 3, "layer": "ground", "roof_of": "den"})), ["Prop 'rug': only a roof prop takes roof_of."]);
    assert_eq!(one(legend(json!(true)), json!({"id": "post", "art": "post", "x": 0, "y": 0, "layer": "over"})), ["Prop 'post': layer must be sort, ground or roof."]);
}

#[test]
fn the_legend_and_boundary_values_are_checked() {
    let mut t = template(json!({"h": {"id": "den", "kind": "safe", "indoor": "yes", "wall_set": "Bad Set"}}), json!([]));
    t.insert("boundary".into(), json!("X"));
    assert_eq!(
        errors_of(&t),
        ["Zone 'h': indoor must be true or false.", "Zone 'h': wall_set 'Bad Set' must be empty or 2-32 characters of a-z, 0-9 and _.", "Boundary 'X' must be empty or 2-32 characters of a-z, 0-9 and _."]
    );
    let mut t = template(legend(json!(true)), json!([roof(5, 2, 5, 4, "den")]));
    t.insert("boundary".into(), json!("zz_brick"));
    assert_eq!(errors_of(&t), Vec::<String>::new());
    t.insert("boundary".into(), json!(""));
    assert_eq!(errors_of(&t), Vec::<String>::new(), "\"\" is the default");
}

// ---- R2 ------------------------------------------------------------------------------------------------------------

#[test]
fn r2_warns_about_indoor_cells_no_roof_covers() {
    let warn = |props: Value| map::roof_warnings(&loaded(&template(legend(json!(true)), props)));
    assert_eq!(warn(json!([roof(5, 2, 5, 4, "den")])), Vec::<String>::new(), "the whole building");
    assert_eq!(warn(json!([roof(6, 3, 3, 2, "den")])), ["Indoor zone 'den': 1 cells lie under no roof prop of the zone, the first at (7, 5)."], "the door left out");
    assert_eq!(warn(json!([roof(5, 2, 2, 4, "den"), {"id": "east_roof", "art": "den_house", "x": 7, "y": 2, "w": 3, "h": 4, "layer": "roof", "roof_of": "den"}])), Vec::<String>::new(), "two roofs share the zone");
    let none = "Indoor zone 'den' has no roof prop (layer roof, roof_of 'den'): its interior shows from outside.";
    assert_eq!(warn(json!([])), [none]);
    assert_eq!(warn(json!([{"id": "post", "art": "post", "x": 5, "y": 2}])), [none], "a standing prop is no roof");
    // an outdoor zone needs no roof; a wall set on it draws nothing
    let outdoor = template(json!({"y": {"id": "yard", "kind": "safe", "wall_set": "interior_stone"}}), json!([]));
    assert_eq!(map::roof_warnings(&loaded(&outdoor)), ["Zone 'y': wall_set 'interior_stone' draws only on an indoor zone (indoor: true)."]);
    assert_eq!(map::roof_warnings(&loaded(&template(legend(json!(false)), json!([])))), ["Zone 'h': wall_set 'zz_plaster' draws only on an indoor zone (indoor: true)."]);
}

#[test]
fn the_catalog_reports_r1_as_an_error_and_r2_as_a_warning() {
    let maps = json!([template(legend(json!(true)), json!([roof(5, 2, 5, 4, "yard")])), {"id": "open_map", "name": "O", "rows": ROWS, "zones": ZONES, "zone_legend": legend(json!(true)), "anchors": [], "paths": {}, "props": [], "tags": []}]);
    let view = View::new(&json!({}), &json!({}), &json!({"maps": maps.as_array().unwrap().iter().map(|m| Value::Object(loaded(json_dict(m)))).collect::<Vec<_>>()}), Layers::default());
    let errors = check::validate_all(&view);
    assert_eq!(errors.get("maps:test_map").cloned().unwrap_or_default(), ["Prop 'den_roof': roof_of 'yard' is not an indoor zone of this map (a zone_legend entry with indoor: true)."]);
    assert!(!errors.contains_key("maps:open_map"), "R2 never blocks");
    let warnings = check::warnings_all(&view);
    assert_eq!(warnings.get("maps:open_map").cloned().unwrap_or_default(), ["Indoor zone 'den' has no roof prop (layer roof, roof_of 'den'): its interior shows from outside."]);
}

#[test]
fn a_map_keeps_its_r2_warning_beside_an_unused_ground_legend_warning() {
    // The roofless den (R2) and a ground legend entry no cell uses (REGIONGROUND1) report on the same `maps:<id>` key.
    let mut t = template(legend(json!(true)), json!([]));
    t.insert("ground".into(), json!(ROWS.iter().map(|r| ".".repeat(r.chars().count())).collect::<Vec<_>>()));
    t.insert("ground_legend".into(), json!({".": "zz_dirt", "f": "zz_boards"}));
    let r2 = map::roof_warnings(&loaded(&t));
    assert!(!r2.is_empty(), "the den has no roof");
    let view = View::new(&json!({}), &json!({}), &json!({"maps": [Value::Object(loaded(&t))]}), Layers::default());
    let found = check::warnings_all(&view).get("maps:test_map").cloned().unwrap_or_default();
    for line in &r2 {
        assert!(found.contains(line), "R2 line {line:?} kept in {found:?}");
    }
    assert!(found.contains(&"Ground legend 'f': no cell uses 'zz_boards'.".to_string()), "{found:?}");
    assert_eq!(found.len(), r2.len() + 1, "{found:?}");
}

fn json_dict(v: &Value) -> &Dict {
    v.as_object().unwrap()
}

// ---- The writer ----------------------------------------------------------------------------------------------------

fn scratch_data(tag: &str, maps: Value) -> PathBuf {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_reveal_{tag}")).until_thread_end();
    let data = t.join("data");
    let _ = fs::remove_dir_all(&data);
    for sub in ["catalog", "worlds", "maps"] {
        fs::create_dir_all(data.join(sub)).unwrap();
    }
    fs::write(data.join("maps/maps.json"), json!({"kind": "map_templates", "version": 1, "entries": maps}).to_string()).unwrap();
    data
}

#[test]
fn the_writer_leaves_every_new_key_out_at_its_default() {
    let plain = json!({"id": "old_map", "name": "Old", "rows": ROWS, "zones": ZONES, "zone_legend": {"y": {"id": "yard", "kind": "safe"}}, "anchors": [], "paths": {}, "props": [{"id": "sign", "art": "signpost", "x": 0, "y": 0}], "tags": []});
    let mut defaults = plain.clone();
    defaults["id"] = json!("default_map");
    defaults["boundary"] = json!("");
    defaults["zone_legend"]["y"]["indoor"] = json!(false);
    defaults["zone_legend"]["y"]["wall_set"] = json!("");
    defaults["props"][0]["roof_of"] = json!("");
    let set = json!({"id": "den_map", "name": "Den", "rows": ROWS, "zones": ZONES, "zone_legend": legend(json!(true)), "anchors": [], "paths": {}, "props": [roof(5, 2, 5, 4, "den")], "boundary": "zz_brick", "tags": []});
    let data = scratch_data("writer", json!([plain, defaults, set]));
    let text = store::resave_text(&data, None, "maps").unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();
    let (a, b, c) = (&doc["entries"][0], &doc["entries"][1], &doc["entries"][2]);
    let keys = |e: &Value| e.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    let stored = ["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "props", "ground", "ground_legend", "ground_patches", "wall_storeys", "tags"];
    assert_eq!(keys(a), stored, "no boundary key at its default");
    assert_eq!(keys(b), stored);
    assert_eq!(b["zone_legend"], a["zone_legend"], "indoor false and wall_set \"\" left out");
    assert_eq!(b["props"], a["props"], "roof_of \"\" left out");
    assert_eq!(a["props"][0].as_object().unwrap().keys().cloned().collect::<Vec<_>>(), source::PROP_KEYS.map(String::from));
    // set values stay, the boundary after the wall storeys, roof_of after the prop's keys
    assert_eq!(keys(c), ["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "props", "ground", "ground_legend", "ground_patches", "wall_storeys", "boundary", "tags"]);
    assert_eq!(c["boundary"], "zz_brick");
    assert_eq!(c["zone_legend"]["h"], json!({"id": "den", "kind": "safe", "indoor": true, "wall_set": "zz_plaster"}));
    assert_eq!(c["props"][0], json!({"id": "den_roof", "art": "den_house", "x": 5, "y": 2, "w": 5, "h": 4, "layer": "roof", "variant": -1, "flip": false, "roof_of": "den"}));
    fs::write(store::file_of(&data, None, "maps"), &text).unwrap();
    assert_eq!(store::resave_text(&data, None, "maps").unwrap(), text, "a second re-save is byte-identical");
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn a_ground_row_holds_indoor_left_out_at_its_default() {
    let data = scratch_data("grounds", json!([]));
    fs::write(data.join(format!("catalog/{GROUNDS_KIND}.json")), json!({"kind": GROUNDS_KIND, "version": 1, "entries": []}).to_string()).unwrap();
    let boards = json!({"id": "test_boards", "name": "Boards", "order": 72, "kind": "land", "colour": "#3a2c22", "wear": 0.06, "strays": false, "tone": 0.08, "patches": 0.0, "marks": [], "marks_per_100": 0, "indoor": true});
    store::upsert(&data, None, GROUNDS_KIND, &dict(boards.clone()), None).expect("an indoor row");
    store::upsert(&data, None, GROUNDS_KIND, &dict(json!({"id": "test_yard", "name": "Yard", "order": 10, "colour": "#2a2622", "indoor": false})), None).expect("an outdoor row");
    let file = store::file_of(&data, None, GROUNDS_KIND);
    let written = fs::read_to_string(&file).unwrap();
    let doc: Value = serde_json::from_str(&written).unwrap();
    assert_eq!(doc["entries"][0], boards, "indoor true kept, last");
    assert!(doc["entries"][1].get("indoor").is_none(), "indoor false left out");
    assert_eq!(store::resave_text(&data, None, GROUNDS_KIND).unwrap(), written);
    let cat = Catalog::from_value(&source::load(&data).catalog);
    assert!(cat.grounds["test_boards"].indoor && !cat.grounds["test_yard"].indoor);
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

// ---- The boundary --------------------------------------------------------------------------------------------------

fn png(path: &Path, w: u32, h: u32) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut b = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    fs::write(path, b).unwrap();
}

#[test]
fn a_location_takes_its_maps_boundary_and_the_dressing_counts_as_its_art() {
    let loc = |b: &str| dict(json!({"id": "bound", "boundary": b}));
    let mut t = template(json!({}), json!([]));
    assert_eq!(art::boundary_of(&loc(""), &t), "");
    t.insert("boundary".into(), json!("zz_brick"));
    assert_eq!(art::boundary_of(&loc(""), &t), "zz_brick", "the map's own");
    assert_eq!(art::boundary_of(&loc("gorge_rock"), &t), "gorge_rock", "the location's first");
    let lineage = vec!["ward".to_string()];
    assert_eq!(art::boundary_dressing_dirs(&lineage, "zz_brick"), ["props/ward/boundary_zz_brick", "props/common/boundary_zz_brick"]);
    let sets = vec!["ward".to_string(), "common".to_string()];
    assert_eq!(art::interior_wall_sets(&sets, "zz_plaster"), ["zz_plaster", "ward", "common"]);
    assert_eq!(art::interior_wall_sets(&sets, ""), sets);
    // A20: the map's boundary warns without art; its dressing is art
    let root = aegis_core::scratch::Scratch::vacant("rpgkit_reveal_a20").until_thread_end();
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let worlds = json!({"test_world": {"manifest": {"id": "test_world"}, "own": {"locations": [{"id": "bound", "map": "test_map", "tileset": "", "markers": [], "boundary": ""}]}}});
    let view = || View::new(&json!({}), &worlds, &json!({"maps": [Value::Object(t.clone())]}), Layers::of(vec![root.to_path_buf()]));
    let a20 = "Location 'bound' boundary 'zz_brick': no art along its tile sets; it draws the set's own walls.";
    assert!(assets::ground_warnings(&view(), "test_world").iter().any(|w| w == a20), "{:?}", assets::ground_warnings(&view(), "test_world"));
    png(&root.join("props/common/boundary_zz_brick/v0.png"), 64, 64);
    assert!(!assets::ground_warnings(&view(), "test_world").iter().any(|w| w == a20));
    let _ = fs::remove_dir_all(&root);
}
