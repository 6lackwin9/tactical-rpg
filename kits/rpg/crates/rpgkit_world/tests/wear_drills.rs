//! UNITDRILL1 world side on fixture rows (no shipped data): `can_wear`, which the party screen's equip paths and the state
//! validation read, passes on the class's training or the member's own drill passives, and on the base unit's
//! `wears_untrained` for that item only; a member's drills come from its character; a built unit's proficiency agrees with
//! the gear rule; a member that loses a drill is refused until the gear comes off.

use rpgkit_rules::stats::Proficiency;
use rpgkit_world::new_game::hero_from_character;
use rpgkit_world::progression::build_with;
use rpgkit_world::state::UnitState;
use rpgkit_world::Data;
use serde_json::json;

fn data() -> Data {
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter", "skill_path": {}, "proficiency": {"sword": {"level": 1}}}],
        "items": [
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword"},
            {"id": "pike", "name": "Pike", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "pike"},
            {"id": "apron", "name": "Apron", "item_type": "ARMOR", "slot": "BODY", "weapon_type": "hide"},
            {"id": "cuirass", "name": "Cuirass", "item_type": "ARMOR", "slot": "BODY", "weapon_type": "plate"}
        ],
        "passives": [
            {"id": "pike_drill", "name": "Pike Drill", "slot": "", "effects": [{"type": "proficiency", "gear": "pike", "levels": 2}]},
            {"id": "pike_stance", "name": "Pike Stance", "slot": "passive", "effects": [{"type": "proficiency", "gear": "pike", "levels": 2}]}
        ],
        "units": [
            {"id": "plain", "name": "Plain", "side": "party", "class_id": "fighter", "level": 1},
            {"id": "piker", "name": "Piker", "side": "party", "class_id": "fighter", "level": 1, "main_hand": "pike"},
            {"id": "aproned", "name": "Aproned", "side": "party", "class_id": "fighter", "level": 1, "body": "apron", "wears_untrained": ["apron"]}
        ]
    });
    let worlds = json!({
        "w": {"manifest": {"id": "w", "start_party": ["hero"]}, "own": {
            "characters": [{"id": "hero", "name": "Hero", "role": "party", "base_unit": "plain", "passives": ["pike_drill"], "_world": "w", "_qid": "w:hero"}]
        }}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn member(base: &str, passives: &[&str]) -> UnitState {
    let mut m = UnitState::create("m", base);
    m.passives = passives.iter().map(|p| p.to_string()).collect();
    m
}

#[test]
fn the_class_trains_and_nothing_else_does_without_a_drill() {
    let d = data();
    let m = member("plain", &[]);
    assert!(m.can_wear(&d, "sword", "main_hand").is_ok());
    assert_eq!(m.can_wear(&d, "pike", "main_hand").unwrap_err(), "Fighter has no training in pike (Pike).");
}

#[test]
fn an_own_drill_trains_its_gear_only() {
    let d = data();
    let m = member("plain", &["pike_drill"]);
    assert!(m.can_wear(&d, "pike", "main_hand").is_ok());
    assert!(m.can_wear(&d, "apron", "body").is_err());
}

#[test]
fn a_slottable_passive_is_not_a_drill() {
    let d = data();
    assert!(member("plain", &["pike_stance"]).can_wear(&d, "pike", "main_hand").is_err());
}

#[test]
fn a_drill_does_not_make_gear_fit_a_wrong_slot() {
    let d = data();
    let err = member("plain", &["pike_drill"]).can_wear(&d, "pike", "body").unwrap_err();
    assert!(err.contains("can't be equipped"), "{err}");
}

#[test]
fn a_hero_gets_its_drills_from_its_character_and_a_member_made_from_the_unit_has_none() {
    let d = data();
    let hero = hero_from_character(&d, "w", "hero").expect("the character can join");
    assert!(hero.can_wear(&d, "pike", "main_hand").is_ok());
    assert!(UnitState::create("plain", "plain").can_wear(&d, "pike", "main_hand").is_err());
}

#[test]
fn wears_untrained_excuses_that_item_on_that_unit_only() {
    let d = data();
    let m = member("aproned", &[]);
    assert!(m.can_wear(&d, "apron", "body").is_ok());
    assert!(m.can_wear(&d, "cuirass", "body").is_err(), "another item of the slot stays refused");
    assert!(m.can_wear(&d, "pike", "main_hand").is_err());
    assert!(member("plain", &[]).can_wear(&d, "apron", "body").is_err(), "another unit stays refused");
}

#[test]
fn the_exception_trains_nothing_in_the_built_unit() {
    let d = data();
    let mut m = member("aproned", &[]);
    m.equipment.insert("body".into(), "apron".into());
    assert_eq!(m.validate(&d), Vec::<String>::new());
    let built = build_with(&d, &m, "w", true).expect("it builds");
    assert_eq!(built.unit.stats.proficiency.level_of("hide"), 0);
}

#[test]
fn a_built_unit_and_the_gear_rule_read_the_same_drills() {
    let d = data();
    let m = member("plain", &["pike_drill"]);
    let built = build_with(&d, &m, "w", true).expect("it builds");
    let class = d.cat.classes.get("fighter").unwrap();
    let rule = Proficiency::from_class(class).with_drills(&d.cat, &m.passives);
    for gear in ["pike", "sword", "hide"] {
        assert_eq!(built.unit.stats.proficiency.level_of(gear), rule.level_of(gear), "{gear}");
    }
    assert!(rule.level_of("pike") >= 1);
}

/// Base gear comes from the unit row, not `equipment`, so validation does not walk it: the member builds, untrained.
#[test]
fn a_member_that_loses_its_drill_keeps_base_gear_untrained() {
    let d = data();
    let mut m = member("piker", &["pike_drill"]);
    assert_eq!(m.validate(&d), Vec::<String>::new());
    assert!(build_with(&d, &m, "w", true).unwrap().unit.stats.proficiency.level_of("pike") >= 1);
    m.passives.clear();
    assert_eq!(m.validate(&d), Vec::<String>::new());
    assert_eq!(build_with(&d, &m, "w", true).expect("it builds").unit.stats.proficiency.level_of("pike"), 0);
}

#[test]
fn a_member_that_loses_its_drill_is_refused_with_the_gear_still_on() {
    let d = data();
    let mut m = member("plain", &["pike_drill"]);
    m.equipment.insert("main_hand".into(), "pike".into());
    assert_eq!(m.validate(&d), Vec::<String>::new());
    m.passives.clear();
    assert_eq!(m.equipment.get("main_hand").map(String::as_str), Some("pike"), "nothing takes the gear off");
    assert!(m.validate(&d).iter().any(|e| e.contains("no training in pike")), "{:?}", m.validate(&d));
    assert!(build_with(&d, &m, "w", true).is_none(), "a member that fails validation does not build");
    m.equipment.shift_remove("main_hand");
    assert_eq!(m.validate(&d), Vec::<String>::new(), "the gear off, it is valid again");
}
