//! PR5-33 (LAYER-UP-DATA): the typed readers of the catalog rows' presentation fields (`rpgkit_data::looks`): a skill's
//! `fx` and `clip`, a weapon family's `fx`, `fx_far`, `attack_clips` and `area_clips` (the catalog's `family_looks`, from
//! its item family rows or `set_family_looks`), a terrain row's `relief`, `relief_depth`, `prop`, `color` and `face`.
//! Every field missing reads as the schema default (empty, 0, flat ground, earth face, no colour). PR5-53 (VFX-DATA1):
//! an effect preset row (`FxPreset`, kind `fx_presets`) and its colours (`parse_rgba`, `rgba_hex`).

use rpgkit_data::looks::{parse_color, parse_rgba, rgba_hex, Face, FxDelivery, FxPreset, FxRole, FxShotLook, FxSlot, Relief, TerrainLook, SPARKS_GRAVITY, SPARKS_SIZE};
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

#[test]
fn an_effect_colour_is_rrggbb_or_rrggbbaa() {
    assert_eq!(parse_rgba("#9ca3af"), Some([0x9c, 0xa3, 0xaf, 255]), "six digits: opaque");
    assert_eq!(parse_rgba("#D9CBB4C8"), Some([0xd9, 0xcb, 0xb4, 0xc8]), "eight digits: the alpha last, any case");
    for bad in ["", "9ca3af", "#9ca3a", "#9ca3af0", "#9ca3afzz", "steel"] {
        assert_eq!(parse_rgba(bad), None, "{bad:?}: the # is needed");
    }
    for c in [[0x9c, 0xa3, 0xaf, 255], [0xd9, 0xcb, 0xb4, 0xc8], [0, 0, 0, 0]] {
        assert_eq!(parse_rgba(&rgba_hex(c)), Some(c), "{c:?} round trips");
    }
    assert_eq!((rgba_hex([0x9c, 0xa3, 0xaf, 255]), rgba_hex([0xd9, 0xcb, 0xb4, 0xc8])), ("#9ca3af".to_string(), "#d9cbb4c8".to_string()));
}

#[test]
fn an_effect_preset_reads_its_parameters_and_defaults_the_rest() {
    let none = FxPreset::from_obj(&obj(json!({"id": "bare"})));
    assert_eq!(none, FxPreset { id: "bare".into(), ..FxPreset::default() }, "no fields: a strike with no delivery and no parts");
    assert_eq!((none.role, none.delivery, none.slot), (FxRole::Strike, FxDelivery::None, FxSlot::None));
    let p = FxPreset::from_obj(&obj(json!({"id": "lob", "role": "strike", "delivery": "shot", "speed": 380.0, "arc": 16.0,
        "shot": {"look": "stone", "size": 2, "color": "#6b7280"},
        "sparks": {"count": 4, "color": "#6b7280", "spread": 120, "speed": 40},
        "ring": {}, "volley": 3, "drop": 48.0, "on_actor": true})));
    assert_eq!((p.delivery, p.speed, p.arc, p.volley, p.drop, p.on_actor), (FxDelivery::Shot, 380.0, 16.0, 3, 48.0, true));
    let shot = p.shot.expect("a projectile");
    assert_eq!((shot.look, shot.size, shot.color, shot.color2, shot.tail), (FxShotLook::Stone, 2.0, [0x6b, 0x72, 0x80, 255], [0x6b, 0x72, 0x80, 255], 0), "color2 left out: color");
    let s = p.sparks.expect("sparks");
    assert_eq!((s.dir, s.gravity, s.size, s.ms, s.foot), (None, SPARKS_GRAVITY, SPARKS_SIZE, 0, false));
    assert_eq!(p.ring, None, "an empty object: no ring");
    for (word, role) in FxRole::IDS.iter().zip([FxRole::Blow, FxRole::Shot, FxRole::Strike, FxRole::Support, FxRole::Status]) {
        assert_eq!((FxRole::parse(word), role.id()), (role, *word));
    }
    for (word, slot) in FxSlot::IDS.iter().zip([FxSlot::None, FxSlot::Head, FxSlot::Body, FxSlot::Feet, FxSlot::Tint, FxSlot::Alpha]) {
        assert_eq!(FxSlot::parse(word), slot, "{word:?}");
    }
    for (word, d) in FxDelivery::IDS.iter().zip([FxDelivery::None, FxDelivery::Melee, FxDelivery::Shot, FxDelivery::Beam, FxDelivery::Cast]) {
        assert_eq!(FxDelivery::parse(word), d, "{word:?}");
    }
    let cat = Catalog::from_value(&json!({"fx_presets": [{"id": "b", "role": "blow"}, {"id": "a", "role": "status", "slot": "alpha"}]}));
    assert_eq!(cat.fx_presets.keys().collect::<Vec<_>>(), ["b", "a"], "keyed by look key in file order");
    assert_eq!((cat.fx_presets["b"].role, cat.fx_presets["a"].slot), (FxRole::Blow, FxSlot::Alpha));
}
