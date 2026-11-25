//! CLASSKEY1 (`PR6-14`, `docs/sessions/U2_CLASS.md` "Class keystones") on a scratch catalog built here (no shipped
//! data): a class's `keystone: {id, level}` goes into the keystone slot of a derived party unit of the class at that
//! level or above whose own deck names no keystone (static party rows and growth members alike), under the deck rule
//! only; a foe of the class takes none; a deck that names a keystone keeps its own; a keystone graft or the wielded
//! family's keystone (mastery 3, a masterwork main hand) takes the one slot instead; a fixed-stat unit and a class
//! without a keystone get none.
use rpgkit_data::{Catalog, Deck};
use rpgkit_rules::gear::Slot;
use rpgkit_rules::ruleset::DeckSlots;
use rpgkit_rules::{build_from, build_unit, build_unit_in, DeckRule, Growth, Unit, UnitBuild};
use serde_json::{json, Value};

/// The level the test class keystones are learned at (the spec's level 6).
const KEY_LEVEL: i32 = 6;

fn rule() -> DeckRule {
    DeckRule { basics: Vec::new(), slots: DeckSlots { active: 2, passive: 1, keystone: 1 } }
}

fn class(id: &str, keystone: Value) -> Value {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let growth = json!({"STR": 1, "DEX": 1, "INT": 1, "CONS": 1, "PERC": 1, "KNOW": 0});
    let mut c = json!({"id": id, "name": id, "base_stats": stats, "growth": growth,
        "proficiency": {"sword": {"level": 2, "scaling": 1.0}}, "skill_path": {"slash": 1}, "main_hand": "w_sword"});
    if !keystone.is_null() {
        c["keystone"] = keystone;
    }
    c
}

fn catalog() -> Catalog {
    Catalog::from_value(&json!({
        "classes": [
            class("guard", json!({"id": "k_guard", "level": KEY_LEVEL})),
            class("scout", json!({"id": "k_scout", "level": KEY_LEVEL})),
            class("plain", Value::Null)
        ],
        "units": [
            {"id": "guard_recruit", "name": "Guard", "class_id": "guard", "level": KEY_LEVEL - 1, "side": "party"},
            {"id": "scout_recruit", "name": "Scout", "class_id": "scout", "level": KEY_LEVEL - 1, "side": "party"},
            {"id": "plain_recruit", "name": "Plain", "class_id": "plain", "level": KEY_LEVEL, "side": "party"},
            {"id": "guard_own", "name": "Own Key", "class_id": "guard", "level": KEY_LEVEL, "side": "party", "deck": {"keystone": ["k_own"]}},
            {"id": "guard_foe", "name": "Foe", "class_id": "guard", "level": KEY_LEVEL, "side": "enemy"},
            {"id": "guard_fixed", "name": "Fixed", "class_id": "guard", "level": KEY_LEVEL, "side": "enemy", "stat_mode": "fixed", "hp": 40, "speed": 10, "attack_range": 1}
        ],
        "skills": [
            {"id": "slash", "name": "Slash", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"},
            {"id": "crown", "name": "Crown", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"},
            {"id": "master_cut", "name": "Master Cut", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"}
        ],
        "passives": [
            {"id": "k_guard", "name": "Guard Key", "slot": "keystone", "effects": [{"type": "stat", "stat": "CONS", "value": 2}]},
            {"id": "k_scout", "name": "Scout Key", "slot": "keystone", "effects": [{"type": "stat", "stat": "DEX", "value": 2}]},
            {"id": "k_own", "name": "Own Key", "slot": "keystone", "effects": [{"type": "stat", "stat": "STR", "value": 2}]}
        ],
        "items": [
            {"id": "w_sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10},
            {"id": "w_master", "name": "Masterwork Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 12, "family_keystone": "master_cut"},
            {"id": "g_crown", "name": "Crown Graft", "item_type": "GRAFT", "tier": 3, "body_role": "class", "keystone": true, "body_effects": [{"type": "skill", "skill": "crown"}]}
        ],
        "body_slots": [{"id": "s_class", "name": "Class", "role": "class"}]
    }))
}

fn row_at(cat: &Catalog, id: &str, level: i32) -> rpgkit_data::UnitDef {
    let mut e = cat.units[id].clone();
    e.level = level;
    e
}

fn static_unit(cat: &Catalog, id: &str, level: i32) -> Unit {
    build_unit_in(cat, &row_at(cat, id, level), 0, Some(&rule()))
}

fn member(cat: &Catalog, id: &str, level: i32, g: Growth) -> Unit {
    let e = row_at(cat, id, level);
    build_from(cat, UnitBuild { growth: Some(g), ..UnitBuild::row(&e, 0, Some(&rule())) }).unit
}

fn holds(u: &Unit, id: &str) -> bool {
    u.passives.iter().any(|p| p == id)
}

#[test]
fn the_class_keystone_is_read_from_the_class_row() {
    let cat = catalog();
    let k = cat.classes["guard"].keystone.clone().expect("guard has a keystone");
    assert_eq!((k.id.as_str(), k.level), ("k_guard", KEY_LEVEL));
    assert!(cat.classes["plain"].keystone.is_none());
}

#[test]
fn a_recruit_below_the_keystone_level_has_no_keystone() {
    let cat = catalog();
    for (id, key) in [("guard_recruit", "k_guard"), ("scout_recruit", "k_scout")] {
        let s = static_unit(&cat, id, KEY_LEVEL - 1);
        assert!(s.deck.keystone.is_empty() && !holds(&s, key), "{id} static: {:?}", s.passives);
        let m = member(&cat, id, KEY_LEVEL - 1, Growth::default());
        assert!(m.deck.keystone.is_empty() && !holds(&m, key), "{id} member: {:?}", m.passives);
    }
}

#[test]
fn a_recruit_at_the_keystone_level_slots_its_class_keystone() {
    let cat = catalog();
    for (id, key) in [("guard_recruit", "k_guard"), ("scout_recruit", "k_scout")] {
        for level in [KEY_LEVEL, KEY_LEVEL + 1] {
            let s = static_unit(&cat, id, level);
            assert_eq!(s.deck.keystone, vec![key.to_string()], "{id} static at {level}");
            assert!(holds(&s, key), "{id} static at {level}: {:?}", s.passives);
            let m = member(&cat, id, level, Growth::default());
            assert_eq!(m.deck.keystone, vec![key.to_string()], "{id} member at {level}");
            assert!(holds(&m, key), "{id} member at {level}: {:?}", m.passives);
        }
    }
    // the keystone's effects apply as a slotted passive's (k_guard: +2 CONS)
    let keyed = static_unit(&cat, "guard_recruit", KEY_LEVEL);
    let mut bare = row_at(&cat, "guard_recruit", KEY_LEVEL);
    bare.deck = Deck { keystone: vec!["k_none".into()], ..Deck::default() };
    let without = build_unit_in(&cat, &bare, 0, Some(&rule()));
    assert_eq!(keyed.stats.stat("CONS", None), without.stats.stat("CONS", None) + 2);
}

#[test]
fn a_deck_naming_a_keystone_keeps_its_own() {
    let cat = catalog();
    let s = static_unit(&cat, "guard_own", KEY_LEVEL);
    assert_eq!(s.deck.keystone, vec!["k_own".to_string()]);
    assert!(holds(&s, "k_own") && !holds(&s, "k_guard"), "{:?}", s.passives);
    // a member's saved deck naming a keystone (owned through its passives)
    let g = Growth { deck: Deck { keystone: vec!["k_own".into()], ..Deck::default() }, ..Growth::default() };
    let e = row_at(&cat, "guard_recruit", KEY_LEVEL);
    let over = rpgkit_rules::Overrides { passives: vec!["k_own".into()], ..Default::default() };
    let m = build_from(&cat, UnitBuild { growth: Some(g), over, ..UnitBuild::row(&e, 0, Some(&rule())) }).unit;
    assert_eq!(m.deck.keystone, vec!["k_own".to_string()]);
    assert!(holds(&m, "k_own") && !holds(&m, "k_guard"), "{:?}", m.passives);
}

#[test]
fn a_keystone_graft_replaces_the_class_keystone() {
    let cat = catalog();
    let g = Growth { body: vec![cat.items["g_crown"].clone()], ..Growth::default() };
    let m = member(&cat, "guard_recruit", KEY_LEVEL, g);
    assert_eq!(m.keystone_skills, vec!["crown".to_string()]);
    assert!(m.deck.keystone.is_empty() && !holds(&m, "k_guard"), "{:?} / {:?}", m.deck.keystone, m.passives);
}

#[test]
fn a_weapon_keystone_replaces_the_class_keystone() {
    let cat = catalog();
    let g = Growth {
        equipment: vec![(Slot::MainHand, cat.items["w_master"].clone())],
        mastery: [("sword".to_string(), rpgkit_data::mastery::MASTERY_MAX)].into_iter().collect(),
        masterwork: true,
        forge_rites: vec!["sword".into()],
        ..Growth::default()
    };
    let m = member(&cat, "guard_recruit", KEY_LEVEL, g);
    assert_eq!(m.mastery_keystone, "master_cut");
    assert!(m.deck.keystone.is_empty() && !holds(&m, "k_guard"), "{:?} / {:?}", m.deck.keystone, m.passives);
}

#[test]
fn no_class_keystone_without_the_deck_rule_a_derived_party_row_or_a_class_keystone() {
    let cat = catalog();
    let classic = build_unit(&cat, &row_at(&cat, "guard_recruit", KEY_LEVEL), 0);
    assert!(classic.deck.keystone.is_empty() && !holds(&classic, "k_guard"));
    let foe = static_unit(&cat, "guard_foe", KEY_LEVEL);
    assert!(foe.deck.keystone.is_empty() && !holds(&foe, "k_guard"), "a foe of the class takes none");
    let fixed = static_unit(&cat, "guard_fixed", KEY_LEVEL);
    assert!(fixed.deck.keystone.is_empty() && !holds(&fixed, "k_guard"));
    let plain = static_unit(&cat, "plain_recruit", KEY_LEVEL);
    assert!(plain.deck.keystone.is_empty());
    // no keystone slot in the ruleset: nothing slotted
    let none = DeckRule { basics: Vec::new(), slots: DeckSlots { active: 2, passive: 1, keystone: 0 } };
    let u = build_unit_in(&cat, &row_at(&cat, "guard_recruit", KEY_LEVEL), 0, Some(&none));
    assert!(u.deck.keystone.is_empty() && !holds(&u, "k_guard"));
}
