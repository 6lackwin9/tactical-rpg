//! PK2-S2 (`docs/sessions/PK2_GRAFTS.md` section 3) on a scratch catalog built here (no shipped data): a member's body
//! grafts (`Growth::body`) apply their `body_effects` as a passive's, so a graft may grant a unit meter (gear still may
//! not); the meter effect's `floors` table gives the floor by the graft count (the last entry past the table), plus the
//! member's floor offset; the graft count limits the channeled marks (`Unit::channeled_limit`); a keystone graft's skill
//! is open without an active slot and takes a keystone slot from the slotted passives. Without a body slot row a member
//! holds no body: no floor table, the ruleset's limit.

use rpgkit_data::{Catalog, Deck};
use rpgkit_rules::gear::Slot;
use rpgkit_rules::grants::{self, Grant};
use rpgkit_rules::ruleset::DeckSlots;
use rpgkit_rules::{build_from, DeckRule, Growth, Overrides, UnitBuild};
use serde_json::{json, Value};

/// The floor table every test graft's meter carries (by graft count 1, 2, 3, 4).
const FLOORS: [i32; 4] = [5, 15, 30, 50];
const RULESET_LIMIT: i32 = 1;

fn graft(id: &str, skill: &str, keystone: bool) -> Value {
    json!({"id": id, "name": id, "item_type": "GRAFT", "tier": if keystone { 3 } else { 1 }, "body_role": if keystone { "class" } else { "monster" }, "keystone": keystone,
        "body_effects": [{"type": "skill", "skill": skill}, {"type": "meter", "meter": "gauge", "floors": FLOORS}]})
}

fn catalog(slots: bool) -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let body_slots = if slots {
        json!([{"id": "s_class", "name": "Class", "role": "class"}, {"id": "s_a", "name": "A", "role": "monster"}, {"id": "s_b", "name": "B", "role": "monster"}, {"id": "s_c", "name": "C", "role": "monster"}])
    } else {
        json!([])
    };
    Catalog::from_value(&json!({
        "classes": [{"id": "adept", "name": "Adept", "base_stats": stats, "growth": {}, "proficiency": {}, "skill_path": {"slash": 1}}],
        "units": [{"id": "adept", "name": "Adept", "class_id": "adept", "level": 1, "side": "player"}],
        "skills": [
            {"id": "slash", "name": "Slash", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"},
            {"id": "claw", "name": "Claw", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "channel": {"only": true}},
            {"id": "bite", "name": "Bite", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "channel": {"only": true}},
            {"id": "sting", "name": "Sting", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "channel": {"only": true}},
            {"id": "crown", "name": "Crown", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "channel": {"only": true}}
        ],
        "passives": [{"id": "key_passive", "name": "Key Passive", "slot": "keystone", "effects": [{"type": "stat", "stat": "CONS", "value": 2}]}],
        "items": [
            graft("g_claw", "claw", false), graft("g_bite", "bite", false), graft("g_sting", "sting", false), graft("g_crown", "crown", true),
            {"id": "gauge_ring", "name": "Gauge Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [{"type": "meter", "meter": "gauge", "floor": 40}]}
        ],
        "unit_meters": [{"id": "gauge", "name": "Gauge", "min": 0, "max": 100, "start": 0}],
        "body_slots": body_slots
    }))
}

fn body(cat: &Catalog, ids: &[&str]) -> Vec<rpgkit_data::ItemDef> {
    ids.iter().map(|i| cat.items[*i].clone()).collect()
}

fn gauge_floor(u: &rpgkit_rules::Unit) -> Vec<Option<i32>> {
    u.passive_meters.iter().filter(|m| m.meter == "gauge").map(|m| m.floor).collect()
}

fn built(cat: &Catalog, ids: &[&str], offset: i32, deck: Option<&DeckRule>) -> rpgkit_rules::Unit {
    let g = Growth { body: body(cat, ids), floor_offset: offset, ..Growth::default() };
    build_from(cat, UnitBuild { growth: Some(g), ..UnitBuild::row(&cat.units["adept"], 0, deck) }).unit
}

#[test]
fn a_meter_effect_reads_its_floors_table() {
    let list = grants::read(&[json!({"type": "meter", "meter": "gauge", "floors": FLOORS})]);
    assert_eq!(list, vec![Grant::GrantPool { meter: "gauge".into(), start: -1, floor: None, floors: FLOORS.to_vec() }]);
    assert!(!list[0].gear_ok(), "gear still grants no meter");
}

/// One floor per graft count from the table (every graft names the same table, so every grant holds that floor);
/// past the table the last entry holds.
#[test]
fn the_floor_follows_the_graft_count() {
    let cat = catalog(true);
    let order = ["g_claw", "g_bite", "g_sting", "g_crown"];
    for n in 1..=order.len() {
        let u = built(&cat, &order[..n], 0, None);
        assert_eq!(u.graft_count, Some(n as i32));
        let want = FLOORS[n - 1];
        assert_eq!(gauge_floor(&u), vec![Some(want); n], "{n} grafts");
        assert_eq!(u.table_floor(&FLOORS[..2]), Some(FLOORS[(n - 1).min(1)]), "{n} grafts on a 2-entry table");
    }
    let none = built(&cat, &[], 0, None);
    assert_eq!((none.graft_count, gauge_floor(&none)), (Some(0), vec![]), "no graft: no meter");
    assert_eq!(none.table_floor(&FLOORS), None);
}

#[test]
fn the_floor_offset_moves_the_table_floor() {
    let cat = catalog(true);
    let offset = -(FLOORS[0] + 1);
    let u = built(&cat, &["g_claw", "g_bite"], offset, None);
    assert_eq!(gauge_floor(&u), vec![Some(FLOORS[1] + offset); 2]);
    assert_eq!(u.floor_offset, offset);
}

/// A body item applies as a passive (its skill learned, its meter held); without a body slot row the build reads no
/// count (no table floor) and the ruleset's limit stands.
#[test]
fn the_body_needs_a_body_slot_row() {
    let cat = catalog(true);
    let u = built(&cat, &["g_claw"], 0, None);
    assert!(u.owns_skill(&cat, "claw"), "the graft's skill is learned");
    assert_eq!(u.channeled_limit(RULESET_LIMIT), 1);
    let bare = catalog(false);
    let b = built(&bare, &["g_claw"], 0, None);
    assert_eq!(b.graft_count, None);
    assert!(gauge_floor(&b).is_empty(), "no body without a body slot row");
    assert_eq!(b.channeled_limit(RULESET_LIMIT), RULESET_LIMIT);
}

#[test]
fn the_channeled_limit_is_one_per_graft() {
    let cat = catalog(true);
    let order = ["g_claw", "g_bite", "g_sting", "g_crown"];
    for n in 0..=order.len() {
        assert_eq!(built(&cat, &order[..n], 0, None).channeled_limit(RULESET_LIMIT), n as i32, "{n} grafts");
    }
}

/// A worn item's meter is still refused (gear), so only the body's floor table counts.
#[test]
fn a_worn_item_still_grants_no_meter() {
    let cat = catalog(true);
    let g = Growth { equipment: vec![(Slot::Accessory1, cat.items["gauge_ring"].clone())], ..Growth::default() };
    let u = build_from(&cat, UnitBuild { growth: Some(g), ..UnitBuild::row(&cat.units["adept"], 0, None) }).unit;
    assert!(u.inventory.get(Slot::Accessory1).is_some());
    assert!(gauge_floor(&u).is_empty());
}

/// Under the deck rule a keystone graft's skill is open without an active slot and takes the one keystone slot, so a
/// keystone passive the deck names no longer applies.
#[test]
fn a_keystone_graft_takes_the_keystone_slot() {
    let cat = catalog(true);
    let rule = DeckRule { basics: Vec::new(), slots: DeckSlots { active: 1, passive: 0, keystone: 1 } };
    let deck = Deck { keystone: vec!["key_passive".into()], active: vec!["claw".into()], ..Deck::default() };
    let make = |ids: &[&str]| {
        let g = Growth { body: body(&cat, ids), deck: deck.clone(), ..Growth::default() };
        let over = Overrides { passives: vec!["key_passive".into()], ..Overrides::default() };
        build_from(&cat, UnitBuild { growth: Some(g), over, ..UnitBuild::row(&cat.units["adept"], 0, Some(&rule)) }).unit
    };
    let plain = make(&["g_claw"]);
    assert!(plain.passives.contains(&"key_passive".to_string()), "the keystone slot holds the passive");
    assert!(plain.keystone_skills.is_empty());
    let crowned = make(&["g_claw", "g_crown"]);
    assert_eq!(crowned.keystone_skills, vec!["crown".to_string()]);
    assert!(!crowned.passives.contains(&"key_passive".to_string()), "the keystone graft took the slot");
    assert!(crowned.knows_skill(&cat, "crown"), "open without an active slot");
    assert_eq!(crowned.active_families(), vec!["claw".to_string()], "the keystone skill takes no active slot");
}
