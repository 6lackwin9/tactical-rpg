//! REGIONGROUND1-K (docs/specs/REGIONGROUND1_KIT.md, the game part's REGIONGROUND1_SCENE.md): the `grounds`
//! kind (schema, row limits, loader, writer), the template fields and checks G1-G8, the scene's additions (G9 wall
//! storeys, G10 material family, a location's boundary), the lookups, the plan, and the asset checks A16-A20 with the A13
//! and A15 changes. Scratch folders and test-only rows; expectations follow from the spec's rules and messages.

use aegis_content::store::Hooks;
use aegis_core::assets::Layers;
use aegis_core::json::{self, Dict};
use indexmap::IndexMap;
use rpgkit_content::check::{self, assets, ground, View};
use rpgkit_content::ground::{plan, seed_of};
use rpgkit_content::{art, source, store};
use rpgkit_data::{Catalog, GroundDef, GROUNDS_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn scratch(tag: &str) -> aegis_core::scratch::Scratch {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_ground_{tag}"));
    let _ = fs::remove_dir_all(&t);
    fs::create_dir_all(&t).unwrap();
    t
}

fn png(path: &Path, w: u32, h: u32) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut b = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    b.extend(w.to_be_bytes());
    b.extend(h.to_be_bytes());
    fs::write(path, b).unwrap();
}

/// A whole ground row.
fn row(id: &str, order: i64, kind: &str) -> Value {
    json!({"id": id, "name": id, "order": order, "kind": kind, "colour": "#2a2622"})
}

fn rows() -> Value {
    json!([row("dust", 10, "land"), row("road", 60, "land"), row("pool", 5, "water")])
}

/// 12 x 8: walls round the edge, two `~` cells, open floor elsewhere.
const MAP: [&str; 8] = ["############", "#..........#", "#..........#", "#..........#", "#..........#", "#..~~......#", "#..........#", "############"];

/// `MAP`'s ground: `a` everywhere, `w` on the `~` cells; a right ground for the map.
fn ground_rows() -> Vec<String> {
    MAP.iter().map(|r| r.chars().map(|c| if c == '~' { 'w' } else { 'a' }).collect()).collect()
}

fn template(ground: Value, legend: Value, patches: Value) -> Dict {
    dict(json!({"id": "test_map", "name": "T", "rows": MAP, "zones": [], "zone_legend": {}, "anchors": [], "paths": {}, "props": [],
        "ground": ground, "ground_legend": legend, "ground_patches": patches, "wall_storeys": [], "tags": []}))
}

fn good_legend() -> Value {
    json!({"a": "dust", "w": "pool"})
}

fn view_of(maps: Value, worlds: Value, root: Option<&Path>) -> View {
    let layers = root.map_or_else(Layers::default, |r| Layers::of(vec![r.to_path_buf()]));
    View::new(&json!({GROUNDS_KIND: rows()}), &worlds, &json!({"maps": maps}), layers)
}

fn errors_of(t: &Dict) -> Vec<String> {
    ground::map_errors(&view_of(json!([]), json!({}), None), t)
}

fn grounds() -> IndexMap<String, GroundDef> {
    Catalog::from_value(&json!({GROUNDS_KIND: rows()})).grounds
}

// ---- The kind ----------------------------------------------------------------------------------------------------

#[test]
fn the_schema_holds_the_table_with_its_defaults() {
    let fields = aegis_content::schema::fields(source::schema(), "content", GROUNDS_KIND);
    let keys: Vec<String> = fields.iter().map(|f| json::s(json::dict(f), "key")).collect();
    assert_eq!(keys, strs(&["id", "name", "order", "kind", "colour", "wear", "strays", "tone", "patches", "marks", "marks_per_100", "origin"]));
    let default_of = |k: &str| fields.iter().find(|f| f["key"] == k).unwrap()["default"].clone();
    assert_eq!(default_of("kind"), "land");
    assert_eq!(default_of("wear"), 0.1);
    assert_eq!(default_of("tone"), 0.13);
    assert_eq!(default_of("strays"), false);
    assert_eq!(default_of("marks"), json!([]));
    assert_eq!(default_of("marks_per_100"), 0);
    assert_eq!(default_of("origin"), "system");
    let zones = aegis_content::schema::fields(source::schema(), "content", rpgkit_data::ZONES_KIND);
    assert_eq!(fields.iter().find(|f| f["key"] == "origin"), zones.iter().find(|f| f["key"] == "origin"), "the same field as the other Forge kinds");
    assert_eq!(source::GROUNDS_KIND, GROUNDS_KIND);
    assert!(store::RpgContent.kinds(None).contains(&GROUNDS_KIND), "the writer takes the kind");
    assert!(fields.iter().all(|f| f.get("min").is_none() && f.get("max").is_none()), "ranges live in the checks, so normalizing never clamps");
}

#[test]
fn a_right_row_passes_and_each_limit_gives_its_message() {
    let v = view_of(json!([]), json!({}), None);
    let check = |extra: Value| {
        let mut e = dict(row("ash", 500, "land"));
        for (k, x) in extra.as_object().unwrap() {
            e.insert(k.clone(), x.clone());
        }
        check::validate_entry(&v, GROUNDS_KIND, &e, "")
    };
    // the ends of each range pass (a stored float is snapped to 4 places)
    for good in [json!({}), json!({"wear": 0.3, "tone": 0.2, "patches": 0.3, "strays": true}), json!({"wear": 0, "tone": 0, "patches": 0}), json!({"order": 0}), json!({"order": 999}),
        json!({"marks": ["footprints", "scuff", "rut_short", "drag_mark"], "marks_per_100": 10}), json!({"kind": "water"}), json!({"id": "a1"}), json!({"id": "a".repeat(24)}), json!({"name": "n".repeat(32)})] {
        assert_eq!(check(good.clone()), Vec::<String>::new(), "{good}");
    }
    let n33 = "n".repeat(33);
    let id25 = "a".repeat(25);
    let id_rule = |id: &str| format!("ID '{id}' must be 2-24 characters of a-z, 0-9 and _.");
    let cases: Vec<(Value, String)> = vec![
        (json!({"id": ""}), "ID is required.".into()),
        (json!({"id": "a"}), id_rule("a")),
        (json!({"id": "Bad-Id"}), id_rule("Bad-Id")),
        (json!({"id": id25}), id_rule(&id25)),
        (json!({"name": ""}), "Name is required.".into()),
        (json!({"name": n33}), "Name is 33 characters (max 32).".into()),
        (json!({"order": 1000}), "Order must be a whole number 0-999.".into()),
        (json!({"order": -1}), "Order must be a whole number 0-999.".into()),
        (json!({"kind": "lava"}), "Kind: 'lava' is not one of land, water.".into()),
        (json!({"colour": "red"}), "Colour 'red' must be #rrggbb.".into()),
        (json!({"colour": "#12345"}), "Colour '#12345' must be #rrggbb.".into()),
        (json!({"wear": 0.31}), "Wear must be 0-0.3.".into()),
        (json!({"wear": -0.01}), "Wear must be 0-0.3.".into()),
        (json!({"tone": 0.21}), "Tone must be 0-0.2.".into()),
        (json!({"patches": 0.31}), "Patches must be 0-0.3.".into()),
        (json!({"marks": ["aa", "bb", "cc", "dd", "ee"]}), "At most 4 marks (got 5).".into()),
        (json!({"marks": ["Bad-Art"]}), "Mark 'Bad-Art' must be a prop art id: lowercase [a-z0-9_], 2-32 characters.".into()),
        (json!({"marks_per_100": 11}), "Marks per 100 cells must be a whole number 0-10.".into()),
        (json!({"marks_per_100": -1}), "Marks per 100 cells must be a whole number 0-10.".into()),
        (json!({"glow": 1}), "Unknown field 'glow'.".into()),
        (json!({"origin": "mine"}), "Origin: 'mine' is not one of system, user.".into()),
    ];
    for (extra, message) in cases {
        assert_eq!(check(extra.clone()), [message], "{extra}");
    }
    // required keys have no working default
    let bare = |e: Value| check::validate_entry(&v, GROUNDS_KIND, &dict(e), "");
    assert_eq!(bare(json!({"id": "ash", "name": "Ash", "colour": "#000000"})), ["Order must be a whole number 0-999."]);
    assert_eq!(bare(json!({"id": "ash", "name": "Ash", "order": 1})), ["Colour '' must be #rrggbb."]);
    // across rows, the row's old self taken out
    assert_eq!(check::validate_entry(&v, GROUNDS_KIND, &dict(row("dust", 500, "land")), ""), ["ID 'dust' is already used by another ground."]);
    assert_eq!(check::validate_entry(&v, GROUNDS_KIND, &dict(row("ash", 10, "land")), ""), ["Order 10 is already used by ground 'dust'."]);
    assert_eq!(check::validate_entry(&v, GROUNDS_KIND, &dict(row("dust", 10, "land")), "dust"), Vec::<String>::new());
}

/// A scratch data folder holding `grounds` rows and `maps`.
fn data_with(tag: &str, grounds: Value, maps: Value) -> PathBuf {
    let t = scratch(tag).until_thread_end();
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    fs::create_dir_all(data.join("maps")).unwrap();
    fs::write(data.join(format!("catalog/{GROUNDS_KIND}.json")), json!({"kind": GROUNDS_KIND, "version": 1, "entries": grounds}).to_string()).unwrap();
    fs::write(data.join("maps/maps.json"), json!({"kind": "map_templates", "version": 1, "entries": maps}).to_string()).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

#[test]
fn the_loader_keeps_right_rows_and_a_missing_file_is_an_empty_table() {
    let data = data_with("load", json!([row("dust", 10, "land"), {"id": "bad", "name": "Bad", "order": 2000, "colour": "#000000"}]), json!([]));
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[GROUNDS_KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, ["dust"]);
    assert!(src.errors.iter().any(|e| e.contains("ground 'bad': Order must be a whole number 0-999.")), "{:?}", src.errors);
    let cat = Catalog::from_value(&src.catalog);
    let dust = &cat.grounds["dust"];
    assert_eq!((dust.order, dust.kind.as_str(), dust.strays, dust.marks_per_100), (10, "land", false, 0));
    assert!((dust.wear - 0.1).abs() < 1e-9 && (dust.tone - 0.13).abs() < 1e-9 && dust.patches == 0.0, "defaults");
    assert!(!dust.is_water());
    fs::remove_file(data.join(format!("catalog/{GROUNDS_KIND}.json"))).unwrap();
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[GROUNDS_KIND]).is_empty());
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{GROUNDS_KIND}.json"))), "{:?}", src.errors);
    drop_scratch(&data);
}

#[test]
fn the_writer_keeps_rows_and_templates_byte_stable_and_refuses_bad_ones() {
    let plain = json!({"id": "old_map", "name": "Old", "rows": MAP, "zones": [], "zone_legend": {}, "anchors": [], "paths": {}, "props": [], "tags": []});
    let data = data_with("writer", json!([]), json!([plain]));
    let maps = store::file_of(&data, None, "maps");
    let file = store::file_of(&data, None, GROUNDS_KIND);
    assert_eq!(file, data.join(format!("catalog/{GROUNDS_KIND}.json")));

    // a template without the keys normalizes to the empties, in stored order, its other keys as they were
    let text = store::resave_text(&data, None, "maps").unwrap();
    let stored: Value = serde_json::from_str(&text).unwrap();
    let t = stored["entries"][0].as_object().unwrap();
    assert_eq!(t.keys().cloned().collect::<Vec<_>>(), strs(&["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "props", "ground", "ground_legend", "ground_patches", "wall_storeys", "tags"]));
    assert_eq!((&t["ground"], &t["ground_legend"], &t["ground_patches"], &t["wall_storeys"]), (&json!([]), &json!({}), &json!([]), &json!([])));
    assert_eq!(t["rows"], json!(MAP));
    fs::write(&maps, &text).unwrap();
    assert_eq!(store::resave_text(&data, None, "maps").unwrap(), text, "a second re-save is byte-identical");

    // rows: every key in schema order, a re-save identical, numbers as written
    let full = json!({"id": "ash", "name": "Ash", "order": 10, "kind": "land", "colour": "#2a2622", "wear": 0.06, "strays": false, "tone": 0.13, "patches": 0.2, "marks": ["footprints"], "marks_per_100": 2});
    store::upsert(&data, None, GROUNDS_KIND, &dict(full.clone()), None).expect("a right row");
    store::upsert(&data, None, GROUNDS_KIND, &dict(row("pond", 5, "water")), None).expect("a row taking defaults");
    let written = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, GROUNDS_KIND).unwrap(), written, "a re-save is identical");
    let doc: Value = serde_json::from_str(&written).unwrap();
    assert_eq!(doc["entries"][0], full);
    assert_eq!(doc["entries"][1].as_object().unwrap().keys().cloned().collect::<Vec<_>>(), strs(&["id", "name", "order", "kind", "colour", "wear", "strays", "tone", "patches", "marks", "marks_per_100"]));
    // origin: system stays out (the default above), a user row keeps it, and both re-save as they are
    store::upsert(&data, None, GROUNDS_KIND, &dict(json!({"id": "mine", "name": "Mine", "order": 12, "colour": "#000000", "origin": "user"})), None).expect("a user row");
    let with_user = fs::read_to_string(&file).unwrap();
    let doc: Value = serde_json::from_str(&with_user).unwrap();
    assert_eq!(doc["entries"][2]["origin"], "user");
    assert_eq!(store::resave_text(&data, None, GROUNDS_KIND).unwrap(), with_user);
    store::remove(&data, None, GROUNDS_KIND, "mine").expect("the user row goes");
    assert_eq!(fs::read_to_string(&file).unwrap(), written, "upsert + remove leaves the file as it was");
    let refused = store::upsert(&data, None, GROUNDS_KIND, &dict(json!({"id": "slag", "name": "Slag", "order": 11, "colour": "#000000", "wear": 0.5})), None).err().expect("wear .5 is refused");
    assert!(refused.iter().any(|m| m.contains("Wear must be 0-0.3.")), "{refused:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), written, "a refused row leaves the file");

    // a map with ground, through the writer: a patch gets its default, in key order; a re-save is identical
    let mut with_ground = dict(json!({"id": "new_map", "name": "New", "rows": MAP, "anchors": [], "paths": {}, "tags": []}));
    with_ground.insert("ground".into(), json!(ground_rows()));
    with_ground.insert("ground_legend".into(), json!({"a": "ash", "w": "nothing"}));
    let before = fs::read_to_string(&maps).unwrap();
    let refused = store::upsert(&data, None, "maps", &with_ground, None).err().expect("G5");
    assert!(refused.iter().any(|m| m.contains("maps:new_map: Ground legend 'w': no ground 'nothing'.")), "{refused:?}");
    assert_eq!(fs::read_to_string(&maps).unwrap(), before, "a refused map leaves the file");
    with_ground.insert("ground_legend".into(), json!({"a": "ash", "w": "pond"}));
    with_ground.insert("ground_patches".into(), json!([{"y": 2, "x": 6, "h": 1, "w": 2, "id": "stain"}]));
    store::upsert(&data, None, "maps", &with_ground, None).expect("a right map");
    let doc: Value = serde_json::from_str(&fs::read_to_string(&maps).unwrap()).unwrap();
    let patch = doc["entries"][1]["ground_patches"][0].as_object().unwrap();
    assert_eq!(patch.keys().cloned().collect::<Vec<_>>(), source::GROUND_PATCH_KEYS.map(String::from));
    assert_eq!(Value::Object(patch.clone()), json!({"id": "stain", "x": 6, "y": 2, "w": 2, "h": 1, "strength": 0.12}));
    assert_eq!(store::resave_text(&data, None, "maps").unwrap(), fs::read_to_string(&maps).unwrap());

    // a ground a map uses cannot go; once the map is gone it can, and the files come back
    let written = fs::read_to_string(&file).unwrap();
    let refused = store::remove(&data, None, GROUNDS_KIND, "ash").err().expect("in use");
    assert!(refused.iter().any(|m| m.contains("no ground 'ash'")), "{refused:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), written);
    store::remove(&data, None, "maps", "new_map").expect("the map goes");
    store::remove(&data, None, GROUNDS_KIND, "ash").expect("the row goes");
    assert_eq!(fs::read_to_string(&maps).unwrap(), text, "upsert + remove leaves the maps file as it was");
    drop_scratch(&data);
}

// ---- G1-G8 -------------------------------------------------------------------------------------------------------

#[test]
fn a_right_ground_and_an_empty_one_pass() {
    assert_eq!(errors_of(&template(json!(ground_rows()), good_legend(), json!([]))), Vec::<String>::new());
    assert_eq!(errors_of(&template(json!([]), json!({}), json!([]))), Vec::<String>::new());
    let mut absent = template(json!([]), json!({}), json!([]));
    for k in ["ground", "ground_legend", "ground_patches", "wall_storeys"] {
        absent.shift_remove(k);
    }
    assert_eq!(errors_of(&absent), Vec::<String>::new());
    // `#` takes any ground, water too
    let mut walls_of_water = ground_rows();
    walls_of_water[0] = "w".repeat(12);
    assert_eq!(errors_of(&template(json!(walls_of_water), good_legend(), json!([]))), Vec::<String>::new());
}

#[test]
fn g1_legend_and_patches_need_rows() {
    let message = ["Ground legend and patches need ground rows."];
    assert_eq!(errors_of(&template(json!([]), good_legend(), json!([]))), message);
    assert_eq!(errors_of(&template(json!([]), json!({}), json!([{"id": "stain", "x": 1, "y": 1, "w": 1, "h": 1}]))), message);
}

#[test]
fn g2_one_string_per_row_one_char_per_cell() {
    let mut short = ground_rows();
    short.pop();
    assert_eq!(errors_of(&template(json!(short), good_legend(), json!([]))), ["Ground has 12 x 7 cells; the map has 12 x 8."]);
    let mut ragged = ground_rows();
    ragged[3].push('a');
    assert_eq!(errors_of(&template(json!(ragged), good_legend(), json!([]))), ["Ground has 13 x 8 cells; the map has 12 x 8."]);
    let mut not_text = json!(ground_rows());
    not_text[2] = json!(7);
    assert_eq!(errors_of(&template(not_text, good_legend(), json!([]))), ["Ground has 0 x 8 cells; the map has 12 x 8."]);
}

#[test]
fn g3_legend_keys_are_one_lowercase_letter_or_digit() {
    for key in ["A", "ab", "#", ""] {
        let mut legend = good_legend();
        legend[key] = json!("dust");
        assert_eq!(errors_of(&template(json!(ground_rows()), legend, json!([]))), [format!("Ground legend key '{key}' must be one lowercase letter or digit.")]);
    }
    let mut digit = good_legend();
    digit["7"] = json!("dust");
    assert_eq!(errors_of(&template(json!(ground_rows()), digit, json!([]))), Vec::<String>::new());
}

#[test]
fn g4_each_wrong_char_is_named_at_its_first_cell() {
    let mut cells = ground_rows();
    cells[1] = "azzzzzzzzzza".into();
    cells[2] = "ay.aaaaaaaaa".into();
    let found = errors_of(&template(json!(cells), good_legend(), json!([])));
    assert_eq!(found, ["Ground (1, 1): 'z' is not in the legend.", "Ground (1, 2): 'y' is not in the legend.", "Ground (2, 2): '.' is not in the legend."]);
}

#[test]
fn g5_legend_ids_are_rows() {
    assert_eq!(errors_of(&template(json!(ground_rows()), json!({"a": "dust", "w": "nowhere"}), json!([]))), ["Ground legend 'w': no ground 'nowhere'."]);
}

#[test]
fn g6_at_most_twelve_grounds() {
    let mut legend = good_legend();
    for key in "bcdefghijk".chars() {
        legend[key.to_string()] = json!("dust");
    }
    assert_eq!(errors_of(&template(json!(ground_rows()), legend.clone(), json!([]))), Vec::<String>::new(), "12 pass");
    legend["l"] = json!("dust");
    assert_eq!(errors_of(&template(json!(ground_rows()), legend, json!([]))), ["At most 12 grounds per map (got 13)."]);
}

#[test]
fn g7_the_picture_does_not_lie_about_walking() {
    let mut water_on_floor = ground_rows();
    water_on_floor[1] = "awwaaaaaaaaa".into();
    assert_eq!(errors_of(&template(json!(water_on_floor), good_legend(), json!([]))), ["Ground (1, 1): 'pool' on a '.' cell."]);
    let land_on_water = ground_rows().iter().map(|r| r.replace('w', "a")).collect::<Vec<_>>();
    assert_eq!(errors_of(&template(json!(land_on_water), good_legend(), json!([]))), ["Ground (3, 5): 'dust' on a '~' cell."]);
}

#[test]
fn g8_placed_patches() {
    let one = |patch: Value| errors_of(&template(json!(ground_rows()), good_legend(), json!([patch])));
    let at = |id: &str, x: i64, y: i64, w: i64, h: i64| json!({"id": id, "x": x, "y": y, "w": w, "h": h});
    assert_eq!(one(at("stain", 6, 2, 2, 1)), Vec::<String>::new(), "open ground all round; strength takes its default");
    let mut with_strength = at("stain", 6, 2, 2, 1);
    for ok in [0.02, 0.25] {
        with_strength["strength"] = json!(ok);
        assert_eq!(one(with_strength.clone()), Vec::<String>::new());
    }
    let mut keyed = at("stain", 6, 2, 2, 1);
    keyed["colour"] = json!("red");
    assert_eq!(one(keyed), ["Ground patch 'stain': unknown field 'colour'."]);
    assert_eq!(one(at("S", 6, 2, 2, 1)), ["Ground patch 'S': its id must be lowercase [a-z0-9_], 2-24 characters."]);
    let whole = "Ground patch 'stain': x, y, w and h must be whole numbers, w and h at least 1.";
    assert_eq!(one(at("stain", 6, 2, 0, 1)), [whole]);
    assert_eq!(one(json!({"id": "stain", "x": 6, "y": 2, "w": 2})), [whole]);
    assert_eq!(one(at("stain", 11, 2, 3, 1)), ["Ground patch 'stain': it leaves the map (12 x 8)."]);
    assert_eq!(one(at("stain", -1, 2, 2, 1)), ["Ground patch 'stain': it leaves the map (12 x 8)."]);
    for bad in [0.01, 0.3] {
        with_strength["strength"] = json!(bad);
        assert_eq!(one(with_strength.clone()), ["Ground patch 'stain': strength must be 0.02-0.25."]);
    }
    // the ring: a wall beside it, water beside it, two grounds in it
    assert_eq!(one(at("stain", 1, 1, 1, 1)), ["Ground patch 'stain': (0, 0) is a wall."]);
    assert_eq!(one(at("stain", 5, 5, 1, 1)), ["Ground patch 'stain': (4, 5) is water."]);
    let mut mixed = ground_rows();
    mixed[3] = "aaaaaaaaabba".into();
    let legend = json!({"a": "dust", "w": "pool", "b": "road"});
    assert_eq!(errors_of(&template(json!(mixed), legend, json!([at("stain", 8, 2, 1, 1)]))), ["Ground patch 'stain': its rect and a ring of one cell hold both 'dust' and 'road'."]);
    // a list
    let twice = json!([at("stain", 6, 2, 1, 1), at("stain", 6, 3, 1, 1)]);
    assert_eq!(errors_of(&template(json!(ground_rows()), good_legend(), twice)), ["Ground patch 'stain': its id is used twice."]);
    let many: Vec<Value> = (0..=ground::GROUND_PATCHES_MAX).map(|i| at(&format!("p{i}"), 6, 2, 1, 1)).collect();
    assert_eq!(errors_of(&template(json!(ground_rows()), good_legend(), json!(many))), ["At most 16 ground patches per map (got 17)."]);
    assert_eq!(errors_of(&template(json!(ground_rows()), good_legend(), json!("x"))), ["Ground patches must be a list of objects."]);
}

#[test]
fn validate_all_reports_under_the_map_and_an_unused_entry_warns() {
    let t = template(json!(ground_rows()), json!({"a": "dust", "w": "pool", "r": "road"}), json!([]));
    let bad = template(json!(["a"]), good_legend(), json!([]));
    let mut bad = bad.clone();
    bad.insert("id".into(), json!("bad_map"));
    let v = view_of(json!([t, bad]), json!({}), None);
    let problems = check::validate_all(&v);
    assert_eq!(problems.get("maps:bad_map").unwrap(), &["Ground has 1 x 1 cells; the map has 12 x 8."]);
    assert!(!problems.contains_key("maps:test_map"));
    assert_eq!(check::warnings_all(&v).get("maps:test_map").unwrap(), &["Ground legend 'r': no cell uses 'road'."]);
    assert!(ground::legend_warnings(&template(json!(ground_rows()), good_legend(), json!([]))).is_empty());
    assert!(ground::legend_warnings(&template(json!([]), json!({}), json!([]))).is_empty());
}

// ---- Numbers past any map: refused or ignored, never a panic -----------------------------------------------------------

/// x, y, w or h values no map holds: the ends of i64, past them, and a float too large to be one.
fn huge() -> Vec<Value> {
    vec![json!(i64::MAX), json!(i64::MIN), json!(u64::MAX), json!(1e300)]
}

#[test]
fn g8_huge_numbers_are_refused() {
    for n in huge() {
        for key in ["x", "y", "w", "h"] {
            let mut patch = json!({"id": "stain", "x": 6, "y": 2, "w": 2, "h": 1});
            patch[key] = n.clone();
            let found = errors_of(&template(json!(ground_rows()), good_legend(), json!([patch.clone()])));
            assert_eq!(found.len(), 1, "{patch}: {found:?}");
            assert!(found[0].starts_with("Ground patch 'stain': "), "{found:?}");
        }
    }
    // a start and a size each in range alone, whose sum is not
    let found = errors_of(&template(json!(ground_rows()), good_legend(), json!([{"id": "stain", "x": i64::MAX, "y": 2, "w": i64::MAX, "h": 1}])));
    assert_eq!(found, ["Ground patch 'stain': it leaves the map (12 x 8)."]);
}

#[test]
fn g9_huge_numbers_are_refused() {
    for n in huge() {
        for key in ["x", "y", "w", "h", "storeys"] {
            let mut rect = json!({"x": 2, "y": 2, "w": 1, "h": 1, "storeys": 2});
            rect[key] = n.clone();
            let found = storeys(json!([rect.clone()]));
            assert_eq!(found.len(), 1, "{rect}: {found:?}");
            assert!(found[0].starts_with("Wall storeys 0: "), "{found:?}");
        }
    }
    assert_eq!(storeys(json!([{"x": i64::MAX, "y": 2, "w": i64::MAX, "h": 1, "storeys": 2}])), ["Wall storeys 0: it leaves the map (8 x 5)."]);
}

#[test]
fn storeys_at_counts_only_rects_with_right_numbers() {
    let at = |rects: Value| ground::storeys_at(&dict(json!({"wall_storeys": rects})), 3, 2);
    let good = json!({"x": 2, "y": 2, "w": 2, "h": 1, "storeys": 3});
    assert_eq!(at(json!([good])), 3);
    for (key, bad) in [("w", json!(i64::MAX)), ("x", json!(i64::MAX)), ("h", json!(i64::MAX)), ("y", json!(i64::MIN))] {
        let mut rect = good.clone();
        rect[key] = bad;
        // no panic, and a saturated rect that still holds the cell keeps its storeys, which are in range
        assert!((1..=ground::WALL_STOREYS_MAX).contains(&at(json!([rect]))));
    }
    for (key, bad) in [("storeys", json!(i64::MAX)), ("storeys", json!(5)), ("storeys", json!(1)), ("storeys", json!(-3)), ("storeys", json!("3")), ("w", json!(0)), ("w", json!("2")), ("w", json!(1.5)), ("x", json!(null))] {
        let mut rect = good.clone();
        rect[key] = bad.clone();
        assert_eq!(at(json!([rect])), 1, "{key}: {bad}");
    }
    assert_eq!(at(json!(["x", 3])), 1, "entries that are not objects");
    assert_eq!(at(json!("x")), 1, "not a list");
    let mut bad = good.clone();
    bad["storeys"] = json!(i64::MAX);
    assert_eq!(at(json!([bad, good])), 3, "a bad rect does not hide a good one");
}

#[test]
fn a19_takes_huge_numbers_without_a_panic() {
    let root = scratch("a19_huge");
    let line = |a: &str| format!("Map 'test_map' marker anchor '{a}' stands behind a wall; it draws its outline.");
    let found = |t: Value, markers: Vec<Value>| assets::ground_warnings(&world_view(&root, t, "", json!(markers), ""), "test_world");
    // huge rects count for nothing: the walls stay 1 storey, so only the two rows north of them warn
    let touching = vec![marker_on("touching"), marker_on("clear")];
    for rect in [json!({"x": 4, "y": 4, "w": 1, "h": 1, "storeys": i64::MAX}), json!({"x": 4, "y": 4, "w": i64::MAX, "h": i64::MAX, "storeys": i64::MAX}), json!({"x": i64::MAX, "y": i64::MAX, "w": i64::MAX, "h": i64::MAX, "storeys": 2})] {
        assert_eq!(found(behind_map(json!([]), json!([rect.clone()])), touching.clone()), [line("touching")], "{rect}");
    }
    // a huge prop holds no cell; huge anchors are on no row
    let prop = json!([{"id": "hut", "art": "hut", "x": i64::MAX, "y": i64::MAX, "w": 2, "h": 1, "layer": "sort", "variant": -1, "flip": false}]);
    png(&root.join("props/common/hut/v0.png"), 64, 64);
    assert_eq!(found(behind_map(prop, json!([])), touching.clone()), [line("touching")]);
    for (x, y) in [(json!(2), json!(i64::MAX)), (json!(i64::MAX), json!(3)), (json!(2), json!(i64::MIN)), (json!(i64::MIN), json!(3)), (json!(u64::MAX), json!(u64::MAX)), (json!(2), json!(1e300))] {
        let mut t = behind_map(json!([]), json!([]));
        t["anchors"] = json!([{"id": "far", "x": x, "y": y}]);
        assert_eq!(found(t, vec![marker_on("far")]), Vec::<String>::new());
    }
    let _ = fs::remove_dir_all(&root);
}

// ---- The scene's additions: G9, G10, the boundary --------------------------------------------------------------------

fn storeys(list: Value) -> Vec<String> {
    let mut t = template(json!([]), json!({}), json!([]));
    t.insert("rows".into(), json!(["########", "#......#", "#.####.#", "#......#", "########"]));
    t.insert("wall_storeys".into(), list);
    errors_of(&t)
}

#[test]
fn g9_wall_storeys() {
    let rect = |x: i64, y: i64, w: i64, h: i64, s: i64| json!({"x": x, "y": y, "w": w, "h": h, "storeys": s});
    assert_eq!(storeys(json!([])), Vec::<String>::new());
    assert_eq!(storeys(json!([rect(2, 2, 2, 1, 2), rect(4, 2, 2, 1, 4)])), Vec::<String>::new(), "inner walls");
    assert_eq!(storeys(json!([rect(2, 0, 2, 1, 3)])), Vec::<String>::new(), "the north row may be tall");
    assert_eq!(storeys(json!("x")), ["Wall storeys must be a list of objects."]);
    let storey_rule = "Wall storeys 0: storeys must be a whole number 2-4 (a wall is 1 by default).";
    assert_eq!(storeys(json!([rect(2, 2, 1, 1, 1)])), [storey_rule]);
    assert_eq!(storeys(json!([rect(2, 2, 1, 1, 5)])), [storey_rule]);
    assert_eq!(storeys(json!([rect(2, 2, 0, 1, 2)])), ["Wall storeys 0: x, y, w and h must be whole numbers, w and h at least 1."]);
    assert_eq!(storeys(json!([rect(7, 2, 3, 1, 2)])), ["Wall storeys 0: it leaves the map (8 x 5)."]);
    assert_eq!(storeys(json!([rect(2, 1, 1, 1, 2)])), ["Wall storeys 0: (2, 1) is not a wall."]);
    for short in [rect(0, 2, 1, 1, 2), rect(7, 2, 1, 1, 2), rect(2, 4, 1, 1, 2)] {
        assert_eq!(storeys(json!([short])), ["Wall storeys 0: the south row and the side columns stay short."], "{short}");
    }
    assert_eq!(storeys(json!([rect(2, 2, 2, 1, 2), rect(3, 2, 1, 1, 2)])), ["Wall storeys 1: (3, 2) is in another rect."]);
    let mut extra = rect(2, 2, 1, 1, 2);
    extra["art"] = json!("x");
    assert_eq!(storeys(json!([extra])), ["Wall storeys 0: unknown field 'art'."]);
    let t = dict(json!({"wall_storeys": [rect(2, 2, 2, 1, 3)]}));
    assert_eq!((ground::storeys_at(&t, 3, 2), ground::storeys_at(&t, 4, 2), ground::storeys_at(&Dict::new(), 3, 2)), (3, 1, 1));
}

#[test]
fn g10_the_material_family() {
    let check = |family: Value| ground::family_errors(&dict(json!({"material_family": family})));
    assert!(ground::family_errors(&Dict::new()).is_empty(), "not set");
    assert_eq!(check(json!({})), Vec::<String>::new());
    assert_eq!(check(json!({"palette": ["#1e1e23", "#38393c"], "materials": ["rough stone", "iron"]})), Vec::<String>::new());
    assert_eq!(check(json!([])), ["Material family must be an object."]);
    assert_eq!(check(json!({"palette": ["#1e1e23"]})), ["Material family: materials must list 1-8 names."]);
    assert_eq!(check(json!({"materials": ["iron"]})), ["Material family: palette must list 1-8 colours."]);
    assert_eq!(check(json!({"palette": ["red"], "materials": ["iron"]})), ["Material family: colour 'red' must be #rrggbb."]);
    assert_eq!(check(json!({"palette": ["#000000"], "materials": ["m".repeat(33)]})), [format!("Material family: material '{}' must be 1-32 characters.", "m".repeat(33))]);
    assert_eq!(check(json!({"palette": ["#000000"], "materials": ["iron"], "light": 1})), ["Material family: unknown field 'light'."]);
    let nine = vec!["#000000"; 9];
    assert_eq!(check(json!({"palette": nine, "materials": ["iron"]})), ["Material family: palette must list 1-8 colours."]);
}

#[test]
fn the_new_world_fields_stay_out_until_set() {
    let data = data_with("world_fields", json!([]), json!([]));
    let dir = data.join("worlds/test_world");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "test_world", "name": "Test"}).to_string()).unwrap();
    store::upsert(&data, Some("test_world"), "locations", &dict(json!({"id": "plain", "name": "Plain"})), None).expect("a location");
    let locations = store::file_of(&data, Some("test_world"), "locations");
    let plain = fs::read_to_string(&locations).unwrap();
    assert!(!plain.contains("\"boundary\""), "left out at its default");
    store::upsert(&data, Some("test_world"), "locations", &dict(json!({"id": "walled", "name": "Walled", "boundary": "cliffs"})), None).expect("a location with a boundary");
    let doc: Value = serde_json::from_str(&fs::read_to_string(&locations).unwrap()).unwrap();
    assert_eq!(doc["entries"][1]["boundary"], "cliffs");
    assert_eq!(store::resave_text(&data, Some("test_world"), "locations").unwrap(), fs::read_to_string(&locations).unwrap());

    let before = fs::read_to_string(dir.join("world.json")).unwrap();
    assert!(!store::manifest_text(&data, "test_world").unwrap().contains("material_family"), "left out at its default");
    assert!(!before.contains("material_family"));
    let family = json!({"palette": ["#1e1e23"], "materials": ["iron"]});
    store::set_world(&data, "test_world", &dict(json!({"material_family": family})), false).expect("a right family");
    let doc: Value = serde_json::from_str(&fs::read_to_string(dir.join("world.json")).unwrap()).unwrap();
    assert_eq!(doc["material_family"], family);
    let refused = store::set_world(&data, "test_world", &dict(json!({"material_family": {"palette": ["red"], "materials": ["iron"]}})), false).err().expect("G10");
    assert!(refused.iter().any(|m| m.contains("colour 'red' must be #rrggbb")), "{refused:?}");
    drop_scratch(&data);
}

// ---- The lookups and the plan --------------------------------------------------------------------------------------

#[test]
fn the_lookups_walk_the_region_sets() {
    let lineage = strs(&["child", "parent"]);
    let sets = art::region_sets(&lineage, "ward");
    assert_eq!(art::ground_dirs(&sets, "mud"), ["tilesets/ward/ground/mud", "tilesets/child/ground/mud", "tilesets/parent/ground/mud", "tilesets/common/ground/mud"]);
    assert_eq!(art::ground_px(), 256);
    assert_eq!(art::boundary_dirs(&sets[..1], "cliffs", "wall_face"), ["tilesets/ward/boundary/cliffs/wall_face"]);
}

#[test]
fn the_plan() {
    assert_eq!(plan(&template(json!([]), json!({}), json!([])), &grounds()), None, "no ground rows: the map draws as it did");
    let mut no_keys = template(json!([]), json!({}), json!([]));
    no_keys.shift_remove("ground");
    assert_eq!(plan(&no_keys, &grounds()), None);

    // the legend lists road, dust, pool; the layers go by `order`: pool 5, dust 10, road 60
    let mut cells = ground_rows();
    cells[1] = "arrraaaaaaaa".into();
    let t = template(json!(cells), json!({"r": "road", "a": "dust", "w": "pool"}), json!([{"id": "stain", "x": 6, "y": 3, "w": 2, "h": 1, "strength": 0.2}, {"id": "soft", "x": 8, "y": 3, "w": 1, "h": 1}]));
    let p = plan(&t, &grounds()).expect("a plan");
    assert_eq!(p.layers.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(), ["pool", "dust", "road"]);
    assert_eq!((p.cols, p.rows), (12, 8));
    assert_eq!(p.cells.len(), 96);
    let at = |x: usize, y: usize| p.cells[y * 12 + x];
    assert_eq!((at(0, 0), at(1, 1), at(3, 5)), (1, 2, 0));
    assert_eq!(p.blocked.len(), 96);
    let expected: Vec<bool> = MAP.iter().flat_map(|r| r.chars().map(|c| c == '#')).collect();
    assert_eq!(p.blocked, expected);
    assert_eq!(p.placed.len(), 2);
    assert_eq!((p.placed[0].x, p.placed[0].y, p.placed[0].w, p.placed[0].h), (6, 3, 2, 1));
    assert!((p.placed[0].strength - 0.2).abs() < 1e-6 && (p.placed[1].strength - 0.12).abs() < 1e-6);

    // FNV-1a 64's published vectors, and one seed per template id
    assert_eq!(seed_of(""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(seed_of("a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(p.seed, seed_of("test_map"));
    let mut other_cells = t.clone();
    other_cells.insert("ground".into(), json!(ground_rows()));
    assert_eq!(plan(&other_cells, &grounds()).unwrap().seed, p.seed, "the seed follows the id alone");
    let mut renamed = t.clone();
    renamed.insert("id".into(), json!("other_map"));
    assert_ne!(plan(&renamed, &grounds()).unwrap().seed, p.seed);

    // an entry no cell uses adds no layer, so the same indices and picture
    let unused = template(json!(cells), json!({"r": "road", "a": "dust", "w": "pool", "z": "road"}), json!([]));
    assert_eq!(plan(&unused, &grounds()).unwrap().cells, p.cells);
    let only_land = template(json!(ground_rows().iter().map(|r| r.replace('w', "a")).collect::<Vec<_>>()), json!({"a": "dust", "w": "pool"}), json!([]));
    assert_eq!(plan(&only_land, &grounds()).unwrap().layers.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(), ["dust"]);

    // a patch a check refuses plans nothing: never a phantom 1 x 1, never a number cut down to fit an i32
    let with_patch = |patch: Value| plan(&template(json!(ground_rows()), good_legend(), json!([patch])), &grounds());
    assert!(with_patch(json!({"id": "stain", "x": 6, "y": 2, "w": 2, "h": 1})).is_some());
    for bad in [json!(7), json!("stain"), json!({}), json!({"id": "stain", "x": 6, "y": 2, "w": 2}), json!({"id": "stain", "x": 4294967302_i64, "y": 2, "w": 2, "h": 1}), json!({"id": "stain", "x": 6, "y": 2, "w": 4294967298_i64, "h": 1}),
        json!({"id": "stain", "x": 6, "y": 2, "w": i64::MAX, "h": 1}), json!({"id": "stain", "x": i64::MAX, "y": 2, "w": i64::MAX, "h": 1}), json!({"id": "stain", "x": -1, "y": 2, "w": 2, "h": 1}), json!({"id": "stain", "x": 11, "y": 2, "w": 3, "h": 1}),
        json!({"id": "stain", "x": 6, "y": 2, "w": 0, "h": 1}), json!({"id": "stain", "x": 1.5, "y": 2, "w": 2, "h": 1}), json!({"id": "stain", "x": 6, "y": 2, "w": 2, "h": 1, "strength": 0.9}), json!({"id": "stain", "x": 6, "y": 2, "w": 2, "h": 1, "strength": "x"})] {
        assert_eq!(with_patch(bad.clone()), None, "{bad}");
    }
    assert_eq!(plan(&template(json!(ground_rows()), good_legend(), json!("stain")), &grounds()), None, "patches that are not a list");

    // what a check refuses plans nothing (no panic)
    assert_eq!(plan(&template(json!(["aa"]), good_legend(), json!([])), &grounds()), None);
    assert_eq!(plan(&template(json!(ground_rows()), json!({"a": "dust"}), json!([])), &grounds()), None);
    assert_eq!(plan(&template(json!(ground_rows()), json!({"a": "dust", "w": "nowhere"}), json!([])), &grounds()), None);
}

// ---- The asset checks ------------------------------------------------------------------------------------------------

/// One world, one location on `t`, bound to `tileset`, with `markers` and an optional `boundary`.
fn world_view(root: &Path, t: Value, tileset: &str, markers: Value, boundary: &str) -> View {
    let worlds = json!({"test_world": {"manifest": {"id": "test_world"}, "own": {"locations": [{"id": "bound", "map": "test_map", "tileset": tileset, "markers": markers, "boundary": boundary}]}}});
    view_of(json!([t]), worlds, Some(root))
}

fn with_ground() -> Value {
    Value::Object(template(json!(ground_rows()), good_legend(), json!([])))
}

#[test]
fn a16_a_used_ground_without_art_draws_its_colour() {
    let root = scratch("a16");
    let found = assets::ground_warnings(&world_view(&root, with_ground(), "", json!([]), ""), "test_world");
    assert_eq!(found, ["Map 'test_map' ground 'dust': no art for world 'test_world'; it draws its flat colour.", "Map 'test_map' ground 'pool': no art for world 'test_world'; it draws its flat colour."]);
    // along the chain: the location's set, the world, then common
    png(&root.join("tilesets/ward/ground/dust/v0.png"), 256, 256);
    png(&root.join("tilesets/common/ground/pool/v0.png"), 256, 256);
    assert_eq!(assets::ground_warnings(&world_view(&root, with_ground(), "ward", json!([]), ""), "test_world"), Vec::<String>::new());
    assert_eq!(assets::ground_warnings(&world_view(&root, with_ground(), "", json!([]), ""), "test_world"), ["Map 'test_map' ground 'dust': no art for world 'test_world'; it draws its flat colour."], "another location's set does not count");
    // a map without ground asks nothing; no layers, nothing
    let plain = Value::Object(template(json!([]), json!({}), json!([])));
    assert_eq!(assets::ground_warnings(&world_view(&root, plain, "", json!([]), ""), "test_world"), Vec::<String>::new());
    let mut none = world_view(&root, with_ground(), "", json!([]), "");
    none.assets = Layers::default();
    assert!(assets::ground_warnings(&none, "test_world").is_empty());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a17_a_ground_file_is_256_square_and_a15_leaves_it_alone() {
    let root = scratch("a17");
    png(&root.join("tilesets/ward/ground/dust/v0.png"), 256, 256);
    png(&root.join("tilesets/ward/ground/dust/over_pool/m01.png"), 32, 32);
    png(&root.join("tilesets/ward/floor/v0.png"), 32, 32);
    let v = world_view(&root, with_ground(), "ward", json!([]), "");
    assert_eq!(assets::tileset_errors(&v, "test_world"), Vec::<String>::new(), "A15 does not read ground/");
    assert_eq!(assets::ground_errors(&v, "test_world"), Vec::<String>::new(), "a deeper file is not a texture");
    png(&root.join("tilesets/ward/ground/pool/v0.png"), 128, 128);
    png(&root.join("tilesets/common/ground/dust/v0.png"), 64, 64);
    let found = assets::ground_errors(&v, "test_world");
    assert_eq!(found, ["Ground art 'tilesets/common/ground/dust/v0.png' is 64x64 px; it must be 256 x 256.", "Ground art 'tilesets/ward/ground/pool/v0.png' is 128x128 px; it must be 256 x 256."]);
    assert_eq!(assets::tileset_errors(&v, "test_world"), Vec::<String>::new());
    assert!(check::world::manifest_errors(&v, "test_world").iter().any(|m| m == &found[0]), "the manifest checks carry it");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a15_takes_the_tall_face_and_the_boundarys_files() {
    let root = scratch("a15");
    png(&root.join("tilesets/ward/floor/v0.png"), 32, 32);
    png(&root.join("tilesets/ward/wall_face/v0.png"), 128, 48);
    png(&root.join("tilesets/ward/boundary/cliffs/wall/v0.png"), 32, 24);
    png(&root.join("tilesets/ward/boundary/cliffs/wall_face/v0.png"), 128, 48);
    let v = world_view(&root, with_ground(), "ward", json!([]), "cliffs");
    assert_eq!(assets::tileset_errors(&v, "test_world"), Vec::<String>::new());
    png(&root.join("tilesets/ward/boundary/cliffs/wall_face/v1.png"), 32, 32);
    png(&root.join("tilesets/ward/boundary/cliffs/wall/v1.png"), 128, 48);
    assert_eq!(
        assets::tileset_errors(&v, "test_world"),
        ["Tile art 'tilesets/ward/boundary/cliffs/wall/v1.png' is 128x48 px; a region tile wants 32x32 or 32x24.", "Tile art 'tilesets/ward/boundary/cliffs/wall_face/v1.png' is 32x32 px; a region wall face wants 32x16 or 32x12 or 128x48."]
    );
    let bad = world_view(&root, with_ground(), "", json!([]), "Bad-Id");
    assert_eq!(assets::tileset_errors(&bad, "test_world"), ["Location 'bound' boundary 'Bad-Id' must be 2-32 characters of a-z, 0-9 and _."]);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a13_a_set_may_hold_only_ground_when_its_map_has_ground() {
    let root = scratch("a13");
    let no_set = "Location 'bound' tile set 'ward': no file tilesets/ward/floor/v0.png in any layer; it draws today's sets.";
    assert_eq!(assets::tileset_warnings(&world_view(&root, with_ground(), "ward", json!([]), ""), "test_world"), [no_set]);
    png(&root.join("tilesets/ward/ground/dust/v0.png"), 256, 256);
    // the map's walls still want the set's wall; `.` and `~` come from its grounds
    let wall = "Location 'bound' tile set 'ward': its map uses '#' but tilesets/ward/wall/v0.png is missing; that tile draws today's set.";
    assert_eq!(assets::tileset_warnings(&world_view(&root, with_ground(), "ward", json!([]), ""), "test_world"), [wall]);
    // a map without ground on that set still wants every tile it uses
    let plain = Value::Object(template(json!([]), json!({}), json!([])));
    let found = assets::tileset_warnings(&world_view(&root, plain, "ward", json!([]), ""), "test_world");
    assert_eq!(found.len(), 3, "{found:?}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a18_a_marks_file_is_there_and_small() {
    let root = scratch("a18");
    let worlds = json!({"test_world": {"manifest": {"id": "test_world"}, "own": {"locations": [{"id": "bound", "map": "test_map"}]}}});
    let catalog = json!({GROUNDS_KIND: [{"id": "dust", "name": "Dust", "order": 10, "colour": "#000000", "marks": ["scuff", "rut"]}, row("pool", 5, "water")]});
    png(&root.join("tilesets/common/ground/dust/v0.png"), 256, 256);
    png(&root.join("tilesets/common/ground/pool/v0.png"), 256, 256);
    let view = || View::new(&catalog, &worlds, &json!({"maps": [with_ground()]}), Layers::of(vec![root.to_path_buf()]));
    assert_eq!(assets::ground_warnings(&view(), "test_world"), ["Ground 'dust' mark 'scuff': no file props/{test_world,common}/scuff/v0.png for world 'test_world'.", "Ground 'dust' mark 'rut': no file props/{test_world,common}/rut/v0.png for world 'test_world'."]);
    png(&root.join("props/common/scuff/v0.png"), 64, 64);
    png(&root.join("props/test_world/rut/v0.png"), 64, 32);
    png(&root.join("props/test_world/rut/v1.png"), 65, 32);
    assert_eq!(assets::ground_warnings(&view(), "test_world"), ["Ground 'dust' mark 'rut': props/test_world/rut/v1.png is 65x32 px, larger than 64 x 64."]);
    let _ = fs::remove_dir_all(&root);
}

/// 8 x 8: inner walls at (2..=4, 4); an open north-west cell on the west column, and walls on the south row.
const BEHIND: [&str; 8] = ["########", "..#....#", "#......#", "#......#", "#.###..#", "#......#", "#......#", "########"];

fn behind_map(props: Value, wall_storeys: Value) -> Value {
    let anchors: Vec<Value> = [("touching", 2, 3), ("half", 3, 2), ("clear", 4, 1), ("south", 2, 5), ("by_border", 5, 6), ("west_side", 0, 1), ("unmarked", 3, 3)].iter().map(|(id, x, y)| json!({"id": id, "x": x, "y": y})).collect();
    json!({"id": "test_map", "name": "T", "rows": BEHIND, "zones": [], "zone_legend": {}, "anchors": anchors, "paths": {}, "props": props, "ground": [], "ground_legend": {}, "ground_patches": [], "wall_storeys": wall_storeys, "tags": []})
}

fn marker_on(anchor: &str) -> Value {
    json!({"id": format!("m_{anchor}"), "kind": "point", "anchor": anchor})
}

#[test]
fn a19_a_marker_on_the_two_rows_north_of_a_wall_warns() {
    let root = scratch("a19");
    let markers: Vec<Value> = ["touching", "half", "clear", "south", "by_border", "west_side"].iter().map(|a| marker_on(a)).collect();
    let line = |a: &str| format!("Map 'test_map' marker anchor '{a}' stands behind a wall; it draws its outline.");
    let found = |props: Value, tall: Value| assets::ground_warnings(&world_view(&root, behind_map(props, tall), "", json!(markers.clone()), ""), "test_world");
    // r - 1 and r - 2 warn; r - 3, south of it, by the short south row and on a short side column do not; an anchor
    // without a marker says nothing
    assert_eq!(found(json!([]), json!([])), [line("touching"), line("half")]);
    // a prop without art keeps its block; with art on hand the cell draws no block
    let prop = json!([{"id": "hut", "art": "hut", "x": 2, "y": 4, "w": 2, "h": 1, "layer": "sort", "variant": -1, "flip": false}]);
    assert_eq!(found(prop.clone(), json!([])), [line("touching"), line("half")]);
    png(&root.join("props/common/hut/v0.png"), 64, 64);
    assert_eq!(found(prop, json!([])), Vec::<String>::new());
    // two storeys reach 4 rows: r - 3 now warns as well
    assert_eq!(found(json!([]), json!([{"x": 4, "y": 4, "w": 1, "h": 1, "storeys": 2}])), [line("touching"), line("half"), line("clear")]);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a20_a_boundary_without_art_warns() {
    let root = scratch("a20");
    png(&root.join("tilesets/common/ground/dust/v0.png"), 256, 256);
    png(&root.join("tilesets/common/ground/pool/v0.png"), 256, 256);
    let view = || world_view(&root, with_ground(), "ward", json!([]), "cliffs");
    assert_eq!(assets::ground_warnings(&view(), "test_world"), ["Location 'bound' boundary 'cliffs': no art along its tile sets; it draws the set's own walls."]);
    png(&root.join("tilesets/ward/boundary/cliffs/wall_face/v0.png"), 128, 48);
    assert_eq!(assets::ground_warnings(&view(), "test_world"), Vec::<String>::new());
    let _ = fs::remove_dir_all(&root);
}
