//! PR5-33 (LAYER-UP-DATA): the typed readers of the catalog rows' presentation fields (`rpgkit_data::looks`): a skill's
//! `fx` and `clip`, a weapon family's `fx`, `fx_far`, `attack_clips` and `area_clips` (the catalog's `family_looks`, from
//! its item family rows or `set_family_looks`), a terrain row's `relief`, `relief_depth`, `prop`, `color` and `face`.
//! Every field missing reads as the schema default (empty, 0, flat ground, earth face, no colour).

use rpgkit_data::looks::{parse_color, Face, Relief, TerrainLook};
use rpgkit_data::{Catalog, FamilyLook};
use serde_json::{json, Value};

fn obj(v: Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap()
}

#[test]
fn a_skill_row_reads_its_look_and_clip_and_defaults_to_none() {
    let cat = Catalog::from_value(&json!({"skills": [
        {"id": "lob", "name": "Lob", "kind": "ATTACK", "fx": "fire_lob", "clip": "throw"},
        {"id": "jab", "name": "Jab", "kind": "ATTACK"},
    ]}));
    let get = |id: &str| cat.skills.iter().find(|s| s.id == id).map(|s| (s.fx.clone(), s.clip.clone())).unwrap();
    assert_eq!(get("lob"), ("fire_lob".to_string(), "throw".to_string()));
    assert_eq!(get("jab"), (String::new(), String::new()));
}

#[test]
fn the_family_looks_are_the_item_family_rows_in_file_order() {
    let rows = json!([
        {"id": "pike", "item_type": "WEAPON", "fx": "stab", "fx_far": "far_look", "attack_clips": ["thrust"], "area_clips": ["sweep", "thrust"]},
        {"id": "club", "item_type": "WEAPON"},
    ]);
    let cat = Catalog::from_value(&json!({"item_families": rows}));
    assert_eq!(cat.family_looks.keys().collect::<Vec<_>>(), ["pike", "club"]);
    let pike = FamilyLook { fx: "stab".into(), fx_far: "far_look".into(), attack_clips: vec!["thrust".into()], area_clips: vec!["sweep".into(), "thrust".into()] };
    assert_eq!(cat.family_looks["pike"], pike);
    assert_eq!(cat.family_looks["club"], FamilyLook::default(), "a row without the fields: the defaults");
    let mut other = Catalog::default();
    assert!(other.family_looks.is_empty(), "no rows: no looks");
    let objs: Vec<_> = rows.as_array().unwrap().iter().map(|r| obj(r.clone())).collect();
    other.set_family_looks(objs.iter());
    assert_eq!(other.family_looks, cat.family_looks, "the world data passes the same rows");
}

#[test]
fn a_terrain_row_reads_its_relief_prop_colour_and_face() {
    assert_eq!(TerrainLook::of(None), TerrainLook::default());
    assert_eq!((TerrainLook::default().relief, TerrainLook::default().face, TerrainLook::default().color), (Relief::Ground, Face::Earth, None));
    let sunk = obj(json!({"id": "pit", "relief": "sunken", "relief_depth": 4, "color": "#963214", "face": "lava"}));
    assert_eq!(TerrainLook::of(Some(&sunk)), TerrainLook { relief: Relief::Sunken, depth: 4, prop: String::new(), color: Some([150, 50, 20]), face: Face::Lava });
    let post = obj(json!({"id": "post", "relief": "standing", "relief_depth": 4, "prop": "post"}));
    let look = TerrainLook::of(Some(&post));
    assert_eq!((look.relief, look.depth, look.prop.as_str()), (Relief::Standing, 0, "post"), "only a sunken tile has a depth");
    for (word, relief) in Relief::IDS.iter().zip([Relief::Ground, Relief::Sunken, Relief::Standing, Relief::Block]) {
        assert_eq!(Relief::parse(word), relief, "{word:?}");
    }
    for (word, face) in Face::IDS.iter().zip([Face::Earth, Face::Wall, Face::Water, Face::Lava]) {
        assert_eq!(Face::parse(word), face, "{word:?}");
    }
    assert_eq!((Relief::parse("cliff"), Face::parse("glass")), (Relief::Ground, Face::Earth), "an unknown value is the default");
}

#[test]
fn a_colour_is_six_hex_digits() {
    assert_eq!(parse_color("#3c6838"), Some([0x3c, 0x68, 0x38]));
    assert_eq!(parse_color("3C6838"), Some([0x3c, 0x68, 0x38]), "the # is optional, any case");
    for bad in ["", "#3c683", "#3c68380", "#3c68zz", "red"] {
        assert_eq!(parse_color(bad), None, "{bad:?}");
    }
}
