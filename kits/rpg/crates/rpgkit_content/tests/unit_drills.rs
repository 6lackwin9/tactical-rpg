//! UNITDRILL1 content side on fixture rows (no shipped data), U3.2-S3 (FB-34): training is no longer a gate, so a unit's
//! untrained gear, a character's base unit's and a start party unit's raise no error (the battle applies the wield
//! penalty); the slot rules still hold; `wears_untrained` is retired: no check reads it (the field table still refuses
//! an id that is no item while the field stays in the schema).

use aegis_core::assets::Layers;
use aegis_core::json::Dict;
use rpgkit_content::check::{self, unit, world, View};
use serde_json::{json, Value};

/// Class `fighter` trains `sword`. `pike_drill` is the drill for the `pike` gear; the other passives look like one and
/// are not: `pike_stance` is slottable, `pike_lore` has another effect (with the keys of a drill), `sword_drill` trains another gear.
fn catalog(units: Value) -> Value {
    json!({
        "classes": [{"id": "fighter", "name": "Fighter", "skill_path": {}, "proficiency": {"sword": {"level": 1}}}],
        "items": [
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword"},
            {"id": "pike", "name": "Pike", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "pike"},
            {"id": "apron", "name": "Apron", "item_type": "ARMOR", "slot": "BODY", "weapon_type": "hide"},
            {"id": "ring", "name": "Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY"}
        ],
        "passives": [
            {"id": "pike_drill", "name": "Pike Drill", "slot": "", "effects": [{"type": "proficiency", "gear": "pike", "levels": 2}]},
            {"id": "pike_stance", "name": "Pike Stance", "slot": "passive", "effects": [{"type": "proficiency", "gear": "pike", "levels": 2}]},
            {"id": "pike_lore", "name": "Pike Lore", "slot": "", "effects": [{"type": "ap_regen", "value": 1, "gear": "pike", "levels": 2}]},
            {"id": "sword_drill", "name": "Sword Drill", "slot": "", "effects": [{"type": "proficiency", "gear": "sword", "levels": 1}]}
        ],
        "units": units
    })
}

fn unit_row(id: &str, gear: Value) -> Value {
    let mut row = json!({"id": id, "name": id, "side": "enemy", "class_id": "fighter", "level": 1, "deck": {}, "tactics": []});
    for (k, x) in gear.as_object().unwrap() {
        row[k] = x.clone();
    }
    row
}

/// World `w` with `kinds` (each list of rows gets its `_world` and `_qid`) and the `start_party`.
fn view(units: Value, kinds: Value, start_party: Value) -> View {
    let mut own = serde_json::Map::new();
    for (kind, rows) in kinds.as_object().unwrap() {
        let mut list = rows.as_array().unwrap().clone();
        for r in &mut list {
            let id = r["id"].as_str().unwrap().to_string();
            r["_world"] = json!("w");
            r["_qid"] = json!(format!("w:{id}"));
        }
        own.insert(kind.clone(), Value::Array(list));
    }
    let worlds = json!({"w": {"manifest": {"id": "w", "start_party": start_party}, "own": Value::Object(own)}});
    View::new(&catalog(units), &worlds, &json!({}), Layers::default())
}

fn character(id: &str, base: &str, fields: Value) -> Value {
    let mut c = json!({"id": id, "name": id, "role": "enemy", "base_unit": base});
    for (k, x) in fields.as_object().unwrap() {
        c[k] = x.clone();
    }
    c
}

fn with_characters(gear: Value, characters: Value) -> View {
    view(json!([unit_row("u", gear)]), json!({"characters": characters}), json!([]))
}

fn errors(v: &View) -> Vec<String> {
    unit::unit_errors(v, v.get("units", "u"))
}

fn character_errors(v: &View, id: &str) -> Vec<String> {
    let e: Dict = v.worlds.own_entries("w", "characters").iter().find(|c| c["id"] == id).unwrap().clone();
    world::validate_entry(v, "w", "characters", &e, id)
}

#[test]
fn untrained_gear_raises_no_error() {
    for gear in [json!({"main_hand": "pike"}), json!({"main_hand": "pike", "body": "apron"}), json!({"main_hand": "sword"})] {
        let v = with_characters(gear.clone(), json!([]));
        assert_eq!(errors(&v), Vec::<String>::new(), "{gear}");
    }
}

#[test]
fn the_slot_rules_still_hold() {
    let v = with_characters(json!({"body": "pike"}), json!([]));
    assert_eq!(errors(&v), vec!["Pike can't be equipped in Body."]);
}

#[test]
fn a_character_on_an_untrained_base_unit_raises_no_error() {
    let v = with_characters(json!({"main_hand": "pike"}), json!([character("drilled", "u", json!({"passives": ["pike_drill"]})), character("raw", "u", json!({}))]));
    assert_eq!(errors(&v), Vec::<String>::new());
    assert_eq!(character_errors(&v, "drilled"), Vec::<String>::new());
    assert_eq!(character_errors(&v, "raw"), Vec::<String>::new());
}

#[test]
fn wears_untrained_is_read_by_no_check() {
    let cases = [
        json!({"main_hand": "sword", "wears_untrained": ["apron"]}),
        json!({"main_hand": "sword", "wears_untrained": ["sword"]}),
        json!({"body": "apron", "wears_untrained": ["apron", "apron"]}),
    ];
    for gear in cases {
        let v = with_characters(gear.clone(), json!([]));
        assert!(!errors(&v).iter().any(|e| e.contains("Wears untrained")), "{gear}: {:?}", errors(&v));
    }
}

#[test]
fn the_field_table_refuses_an_unknown_item() {
    let v = with_characters(json!({"body": "apron", "wears_untrained": ["no_such_item"]}), json!([]));
    let e = v.get("units", "u").clone();
    let all = check::validate_entry(&v, "units", &e, "u");
    assert!(all.iter().any(|x| x.contains("Wears untrained") && x.contains("no_such_item")), "{all:?}");
}

#[test]
fn a_start_party_unit_with_untrained_gear_may_stand_there() {
    let party = |id: &str, main: &str| {
        let mut r = unit_row(id, json!({"main_hand": main}));
        r["side"] = json!("party");
        r
    };
    let chars = json!({"characters": [character("hero", "u", json!({"role": "party", "passives": ["pike_drill"]}))]});
    for start in [json!(["u"]), json!(["hero"])] {
        let v = view(json!([party("u", "pike")]), chars.clone(), start.clone());
        assert!(!world::manifest_errors(&v, "w").iter().any(|e| e.contains("Start party")), "{start}: {:?}", world::manifest_errors(&v, "w"));
    }
}
