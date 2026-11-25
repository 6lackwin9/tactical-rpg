//! GEAR-REWORK1 (PR5-51a, `docs/sessions/U3_GEAR_REWORK.md` 1) world side on a scratch world (no shipped data): a
//! member of a class that tags its proficiency entries with roles is under the strict gear rule. `can_wear` (the party
//! screen's equip paths and the state validation) refuses gear at proficiency 0 and gear whose raw `required_stats`
//! exceed the member's unscaled stats plus the proficiency discount; a saved piece it refuses moves to the stash at load
//! with one warning (`save::decode`, `UnitState::sanitize_loadout`). A member of a class without roles (today's catalog)
//! wears both as before (the soft wield penalty). Weapon mastery XP goes to main-hand weapon families only, at
//! proficiency 2 (x1) and 3 (x1.5); never a proficiency 1 family, an off-hand piece or armour.

use rpgkit_data::mastery::{scaled, USE_XP};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{mastery, save, Data, GameState};
use serde_json::json;

fn data() -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let prof = |roles: bool| {
        let e = |level: i32, role: &str| if roles { json!({"level": level, "role": role}) } else { json!({"level": level}) };
        json!({"sword": e(2, "secondary"), "axe": e(3, "main"), "mace": e(1, "minor"), "shield": e(3, "main"), "hide": e(2, "third")})
    };
    let catalog = json!({
        "classes": [
            {"id": "strict", "name": "Strict", "skill_path": {}, "base_stats": stats, "proficiency": prof(true)},
            {"id": "soft", "name": "Soft", "skill_path": {}, "base_stats": stats, "proficiency": prof(false)}
        ],
        "items": [
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword"},
            {"id": "pike", "name": "Pike", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "pike"},
            {"id": "test_blade", "name": "Test Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "test_family"},
            {"id": "great_sword", "name": "Great Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "required_stats": {"STR": 30}},
            {"id": "ring", "name": "Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "weapon_type": "accessory"}
        ],
        "units": [
            {"id": "strict", "name": "Strict", "side": "party", "class_id": "strict", "level": 1},
            {"id": "soft", "name": "Soft", "side": "party", "class_id": "soft", "level": 1}
        ]
    });
    let families = json!([
        {"id": "sword", "item_type": "WEAPON", "hands": ["main_1h"]},
        {"id": "axe", "item_type": "WEAPON", "hands": ["main_1h"]},
        {"id": "mace", "item_type": "WEAPON", "hands": ["main_1h"]},
        {"id": "shield", "item_type": "SHIELD", "hands": ["off"]},
        {"id": "hide", "item_type": "ARMOR", "slots": ["HEAD", "BODY"]},
        {"id": "test_family", "name": "Test Family", "item_type": "WEAPON", "hands": ["main_1h"]}
    ]);
    let worlds = json!({"w": {"manifest": {"id": "w", "start_party": ["strict"]}, "own": {}}});
    Data::from_values(&catalog, &worlds, &json!({"item_families": families}))
}

fn envelope(d: &Data, members: Vec<UnitState>) -> String {
    let mut party = PartyState::default();
    party.members = members;
    let gs = GameState::create(d, "w", party);
    json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": gs.to_value()}).to_string()
}

#[test]
fn the_strict_rule_refuses_untrained_and_unfit_gear_and_the_soft_rule_does_not() {
    let d = data();
    let (strict, soft) = (UnitState::create("a", "strict"), UnitState::create("b", "soft"));
    let untrained = strict.can_wear(&d, "pike", "main_hand").expect_err("proficiency 0 locks");
    assert!(untrained.contains("untrained in pike"), "{untrained}");
    let unfit = strict.can_wear(&d, "great_sword", "main_hand").expect_err("STR 30 is past 12 + the discount");
    assert!(unfit.contains("needs STR 30"), "{unfit}");
    assert_eq!(strict.can_wear(&d, "sword", "main_hand"), Ok(()));
    assert_eq!(strict.can_wear(&d, "ring", "accessory_1"), Ok(()), "an accessory needs no proficiency");
    for item in ["pike", "great_sword", "sword"] {
        assert_eq!(soft.can_wear(&d, item, "main_hand"), Ok(()), "{item}: no roles, no lock (the soft wield penalty)");
    }
    assert!(!strict.trains(&d, "pike") && !soft.trains(&d, "pike"), "training reads the same either way");
}

/// PR7-76 (PT-60): the refusal names the gear type in words (its family's name), never its id; `wear_refusal` is the
/// same line, None for an item the member may wear.
#[test]
fn the_refusal_names_the_gear_type_by_its_family_name() {
    let d = data();
    let strict = UnitState::create("a", "strict");
    let why = strict.can_wear(&d, "test_blade", "main_hand").expect_err("proficiency 0 locks");
    assert!(why.contains("untrained in Test Family") && !why.contains("test_family"), "{why}");
    assert_eq!(strict.wear_refusal(&d, &d.cat.items["test_blade"]), Some(why));
    assert_eq!(strict.wear_refusal(&d, &d.cat.items["sword"]), None);
    assert_eq!(d.family_name("test_family"), "Test Family");
    assert_eq!(d.family_name("pike"), "pike", "a type with no family row keeps its id");
}

#[test]
fn a_saved_piece_the_strict_rule_refuses_moves_to_the_stash_at_load() {
    let d = data();
    let mut strict = UnitState::create("a", "strict");
    strict.equipment.insert("main_hand".into(), "pike".into());
    let mut soft = UnitState::create("b", "soft");
    soft.equipment.insert("main_hand".into(), "pike".into());
    let loaded = save::decode(&d, &envelope(&d, vec![strict, soft])).unwrap_or_else(|e| panic!("the save loads: {e:?}"));
    let moved: Vec<&String> = loaded.warnings.iter().filter(|w| w.contains("Moved from the saved gear to the stash")).collect();
    assert_eq!(moved.len(), 1, "one warning, the strict member's: {:#?}", loaded.warnings);
    assert!(moved[0].contains("Unit 'a'") && moved[0].contains("untrained in pike"), "{}", moved[0]);
    let party = &loaded.state.party;
    assert_eq!(party.members[0].equipment.get("main_hand"), None, "taken off");
    assert_eq!(party.stash.get("pike").copied(), Some(1), "in the stash");
    assert_eq!(party.members[1].equipment.get("main_hand").map(String::as_str), Some("pike"), "the soft member keeps it");
}

#[test]
fn mastery_xp_goes_to_main_hand_weapon_families_at_proficiency_2_and_3_only() {
    let d = data();
    for base in ["strict", "soft"] {
        let m = UnitState::create("m", base);
        for (family, want) in [("sword", true), ("axe", true), ("mace", false), ("shield", false), ("hide", false)] {
            assert_eq!(mastery::masterable(&d, &m, family), want, "{base}: {family}");
        }
        assert_eq!(scaled(USE_XP, mastery::proficiency(&d, &m, "sword")), USE_XP, "proficiency 2: x1");
        assert_eq!(scaled(USE_XP, mastery::proficiency(&d, &m, "axe")), (USE_XP as f64 * 1.5).round() as i64, "proficiency 3: x1.5");
        assert_eq!(scaled(USE_XP, mastery::proficiency(&d, &m, "mace")), 0, "proficiency 1: none");
    }
}
