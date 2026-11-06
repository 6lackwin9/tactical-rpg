//! UNITDRILL1 content side on fixture rows (no shipped data): a unit's gear passes when its class or the drill passives of
//! a world character playing as it give the training, or the row names the item in `wears_untrained`; a character needs
//! its own drills; a start party may not list a unit whose gear only a character's drill trains; `wears_untrained` is
//! refused where it is not a real exception. Both sides of each rule.

use aegis_core::assets::Layers;
use aegis_core::json::Dict;
use rpgkit_content::check::{self, unit, world, View};
use serde_json::{json, Value};

const NO_TRAINING: &str = "This class has no training in 'pike' (Pike).";
const NO_TRAINING_APRON: &str = "This class has no training in 'hide' (Apron).";

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
fn neither_class_nor_drill_fails_with_the_message_of_before() {
    let v = with_characters(json!({"main_hand": "pike"}), json!([]));
    assert_eq!(errors(&v), vec![NO_TRAINING]);
}

#[test]
fn the_class_alone_still_trains_its_gear() {
    let v = with_characters(json!({"main_hand": "sword"}), json!([]));
    assert_eq!(errors(&v), Vec::<String>::new());
}

#[test]
fn a_drill_of_a_character_on_the_unit_trains_it_from_each_source_a_character_has() {
    let kinds = [
        json!({"characters": [character("c", "u", json!({"passives": ["pike_drill"]}))]}),
        json!({"characters": [character("c", "u", json!({"faction": "f"}))], "factions": [{"id": "f", "name": "F", "grants": ["pike_drill"]}]}),
        json!({"characters": [character("c", "u", json!({"disciplines": ["d"]}))], "disciplines": [{"id": "d", "name": "D", "passives": ["pike_drill"]}]}),
        json!({"characters": [character("c", "u", json!({"named_items": ["n"]}))], "items": [{"id": "n", "name": "N", "base_item": "ring", "grants": ["pike_drill"]}]}),
    ];
    for k in kinds {
        let v = view(json!([unit_row("u", json!({"main_hand": "pike"}))]), k.clone(), json!([]));
        assert_eq!(errors(&v), Vec::<String>::new(), "{k}");
    }
}

#[test]
fn what_is_not_this_units_drill_does_not_train_it() {
    // not a drill: a slottable passive, another effect, another gear, an unknown id
    for passive in ["pike_stance", "pike_lore", "sword_drill", "no_such_passive"] {
        let v = with_characters(json!({"main_hand": "pike"}), json!([character("c", "u", json!({"passives": [passive]}))]));
        assert_eq!(errors(&v), vec![NO_TRAINING], "{passive}");
    }
    // a drill, on a character of another unit
    let v = view(
        json!([unit_row("u", json!({"main_hand": "pike"})), unit_row("other", json!({"main_hand": "sword"}))]),
        json!({"characters": [character("c", "other", json!({"passives": ["pike_drill"]}))]}),
        json!([]),
    );
    assert_eq!(errors(&v), vec![NO_TRAINING]);
}

#[test]
fn a_drill_for_one_item_leaves_the_other_item_untrained() {
    let v = with_characters(json!({"main_hand": "pike", "body": "apron"}), json!([character("c", "u", json!({"passives": ["pike_drill"]}))]));
    assert_eq!(errors(&v), vec![NO_TRAINING_APRON]);
}

#[test]
fn a_character_needs_its_own_drill_though_another_passes_the_unit() {
    let v = with_characters(
        json!({"main_hand": "pike"}),
        json!([character("drilled", "u", json!({"passives": ["pike_drill"]})), character("raw", "u", json!({}))]),
    );
    assert_eq!(errors(&v), Vec::<String>::new());
    assert_eq!(character_errors(&v, "drilled"), Vec::<String>::new());
    let raw = character_errors(&v, "raw");
    assert_eq!(raw, vec!["Base unit 'u' wears Pike: neither its class nor this character's drills train 'pike'."]);
}

#[test]
fn wears_untrained_excuses_the_named_item_and_only_that_one() {
    let characters = json!([character("c", "u", json!({"passives": ["pike_drill"]}))]);
    let v = with_characters(json!({"main_hand": "pike", "body": "apron", "wears_untrained": ["apron"]}), characters.clone());
    assert_eq!(errors(&v), Vec::<String>::new());
    assert_eq!(character_errors(&v, "c"), Vec::<String>::new());
    // the pike has no drill here, and the apron's exception does not cover it
    let v = with_characters(json!({"main_hand": "pike", "body": "apron", "wears_untrained": ["apron"]}), json!([]));
    assert_eq!(errors(&v), vec![NO_TRAINING]);
    // the default is off
    let v = with_characters(json!({"main_hand": "pike", "body": "apron"}), characters);
    assert_eq!(errors(&v), vec![NO_TRAINING_APRON]);
}

#[test]
fn wears_untrained_is_refused_where_it_is_no_exception() {
    let drilled = json!([character("c", "u", json!({"passives": ["pike_drill"]}))]);
    let cases = [
        // not worn by this row
        (json!({"main_hand": "sword", "wears_untrained": ["apron"]}), json!([]), "Wears untrained: 'apron' is not in this unit's gear."),
        // the class trains it
        (json!({"main_hand": "sword", "wears_untrained": ["sword"]}), json!([]), "Wears untrained: 'sword' needs no exception (its class or a drill trains it)."),
        // a drill trains it
        (json!({"main_hand": "pike", "wears_untrained": ["pike"]}), drilled, "Wears untrained: 'pike' needs no exception (its class or a drill trains it)."),
        // gear with no type needs no training
        (json!({"accessory_1": "ring", "wears_untrained": ["ring"]}), json!([]), "Wears untrained: 'ring' needs no exception (its class or a drill trains it)."),
        // twice
        (json!({"body": "apron", "wears_untrained": ["apron", "apron"]}), json!([]), "Wears untrained lists 'apron' twice."),
    ];
    for (gear, characters, want) in cases {
        let v = with_characters(gear.clone(), characters);
        assert!(errors(&v).iter().any(|e| e == want), "{gear}: {:?}", errors(&v));
    }
    // a unit with no class has no training rule to be excused from (and, U2.2-S2 FB-23, is an error itself)
    let mut classless = unit_row("u", json!({"body": "apron", "wears_untrained": ["apron"], "stat_mode": "fixed"}));
    classless["class_id"] = json!("");
    let v = view(json!([classless]), json!({}), json!([]));
    assert_eq!(errors(&v), vec!["Fixed units need a class too (their base stats are the class's at their level).", "Wears untrained: 'apron' needs no exception (its class or a drill trains it)."]);
}

#[test]
fn the_field_table_refuses_an_unknown_item() {
    let v = with_characters(json!({"body": "apron", "wears_untrained": ["no_such_item"]}), json!([]));
    let e = v.get("units", "u").clone();
    let all = check::validate_entry(&v, "units", &e, "u");
    assert!(all.iter().any(|x| x.contains("Wears untrained") && x.contains("no_such_item")), "{all:?}");
}

#[test]
fn a_start_party_lists_the_character_when_only_a_drill_trains_the_gear() {
    let party = |id: &str, main: &str| {
        let mut r = unit_row(id, json!({"main_hand": main}));
        r["side"] = json!("party");
        r
    };
    let chars = json!({"characters": [character("hero", "u", json!({"role": "party", "passives": ["pike_drill"]}))]});
    let v = view(json!([party("u", "pike")]), chars.clone(), json!(["u"]));
    let listed = world::manifest_errors(&v, "w");
    assert!(listed.iter().any(|e| e == "Start party unit 'u' wears Pike, which only a character's drill trains: list the character."), "{listed:?}");
    let v = view(json!([party("u", "pike")]), chars, json!(["hero"]));
    assert!(!world::manifest_errors(&v, "w").iter().any(|e| e.contains("Start party")), "{:?}", world::manifest_errors(&v, "w"));
    // the class trains it: the unit may stand there itself
    let v = view(json!([party("u", "sword")]), json!({}), json!(["u"]));
    assert!(!world::manifest_errors(&v, "w").iter().any(|e| e.contains("Start party")), "{:?}", world::manifest_errors(&v, "w"));
}
