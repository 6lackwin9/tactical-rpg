//! UNITDRILL1 world side on fixture rows (no shipped data), U3.2-S3 (FB-34, FB-63 (2)): `can_wear`, which the party
//! screen's equip paths and the state validation read, checks the slot only (training is no gate: an untrained item is
//! worn under the wield penalty); `trains` says whether the member wears an item trained, from the class's training or
//! its drill passives as a build re-gathers them (`progression::member_passives`: the character's sources now, then the
//! saved ids), as is deck passive ownership; a built unit's proficiency agrees with the gear rule; `wears_untrained` is
//! retired (read by nothing).

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
            "characters": [{"id": "hero", "name": "Hero", "role": "party", "base_unit": "plain", "passives": ["pike_drill", "pike_stance"], "_world": "w", "_qid": "w:hero"}]
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
    assert!(m.trains(&d, "sword"));
    assert!(!m.trains(&d, "pike"));
}

#[test]
fn untrained_gear_is_worn_and_valid() {
    let d = data();
    let mut m = member("plain", &[]);
    assert!(m.can_wear(&d, "pike", "main_hand").is_ok(), "training is no gate");
    m.equipment.insert("main_hand".into(), "pike".into());
    assert_eq!(m.validate(&d), Vec::<String>::new());
    let built = build_with(&d, &m, "w", true).expect("it builds");
    let pike = built.unit.inventory.main_weapon().expect("the pike is worn");
    assert!(pike.wield_penalty(&built.unit.stats).untrained, "worn untrained");
}

#[test]
fn an_own_drill_trains_its_gear_only() {
    let d = data();
    let m = member("plain", &["pike_drill"]);
    assert!(m.trains(&d, "pike"));
    assert!(!m.trains(&d, "apron"));
}

#[test]
fn a_slottable_passive_is_not_a_drill() {
    let d = data();
    assert!(!member("plain", &["pike_stance"]).trains(&d, "pike"));
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
    assert!(hero.trains(&d, "pike"));
    assert!(!UnitState::create("plain", "plain").trains(&d, "pike"));
}

/// FB-63 (2): training and deck passive ownership read the re-gathered passives, so a drill or a slottable passive the
/// character grants counts although the saved list lacks it; a member with no character owns its saved ids only.
#[test]
fn training_and_deck_ownership_read_the_regathered_passives() {
    let d = data();
    let mut hero = hero_from_character(&d, "w", "hero").expect("the character can join");
    hero.passives.clear();
    assert!(hero.trains(&d, "pike"), "the character's drill, re-gathered");
    hero.deck.passive = vec!["pike_stance".into()];
    let errors = hero.deck_errors(&d, "hero");
    assert!(!errors.iter().any(|e| e.contains("not owned")), "{errors:?}");
    let mut plain = member("plain", &[]);
    plain.deck.passive = vec!["pike_stance".into()];
    assert!(plain.deck_errors(&d, "plain").iter().any(|e| e.contains("passive 'pike_stance' is not owned")), "{:?}", plain.deck_errors(&d, "plain"));
}

#[test]
fn wears_untrained_is_read_by_nothing() {
    let d = data();
    let m = member("aproned", &[]);
    assert!(!m.trains(&d, "apron"), "the retired exception trains nothing");
    let built = build_with(&d, &m, "w", true).expect("it builds");
    assert_eq!(built.unit.stats.proficiency.level_of("hide"), 0);
    let apron = built.unit.inventory.get(rpgkit_rules::gear::Slot::Body).expect("the apron is worn");
    assert!(apron.wield_penalty(&built.unit.stats).untrained, "worn under the penalty like any untrained item");
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

/// A member that loses its drill keeps its gear and stays valid; it builds and wears the gear untrained.
#[test]
fn a_member_that_loses_its_drill_wears_the_gear_untrained() {
    let d = data();
    let mut m = member("plain", &["pike_drill"]);
    m.equipment.insert("main_hand".into(), "pike".into());
    assert_eq!(m.validate(&d), Vec::<String>::new());
    let trained = build_with(&d, &m, "w", true).expect("it builds");
    assert!(!trained.unit.inventory.main_weapon().unwrap().wield_penalty(&trained.unit.stats).untrained);
    m.passives.clear();
    assert_eq!(m.validate(&d), Vec::<String>::new(), "no longer refused");
    let built = build_with(&d, &m, "w", true).expect("it builds");
    assert_eq!(built.unit.stats.proficiency.level_of("pike"), 0);
    assert!(built.unit.inventory.main_weapon().unwrap().wield_penalty(&built.unit.stats).untrained);
}
