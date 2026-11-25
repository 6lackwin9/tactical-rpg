//! PK2-S1 (`docs/sessions/PK2_GRAFTS.md` section 2) on a scratch catalog built here (no shipped data): the member
//! grants of the passive grammar and the member's permanent max HP offset. `forbid_hands` {hands: 2} and
//! `pierce_melee` read into their typed grants, which gear may not carry (`Grant::gear_ok`); a passive applies them to
//! the unit (a worn item's never); `forbids_hands` refuses exactly the two-handed items; a growth unit's max HP is its
//! derived max plus the offset on every sync (at least 1), and it holds its tracks only once the catalog has a track
//! row; a passive granted mid-battle keeps the battle's max HP and HP.

use indexmap::IndexMap;
use rpgkit_data::Catalog;
use rpgkit_rules::gear::Slot;
use rpgkit_rules::grants::{self, Grant};
use rpgkit_rules::unit::forbids_hands;
use rpgkit_rules::{build_from, Growth, Overrides, UnitBuild};
use serde_json::{json, Value};

const OFFSET: i32 = 25;

fn catalog(tracks: Value) -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    Catalog::from_value(&json!({
        "classes": [{"id": "adept", "name": "Adept", "base_stats": stats, "growth": {}, "proficiency": {}, "skill_path": {"slash": 1}}],
        "units": [{"id": "adept", "name": "Adept", "class_id": "adept", "level": 1, "side": "player"}],
        "skills": [{"id": "slash", "name": "Slash", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"}],
        "passives": [
            {"id": "shakes", "name": "Shakes", "slot": "", "effects": [{"type": "forbid_hands", "hands": 2}, {"type": "pierce_melee"}]},
            {"id": "sturdy", "name": "Sturdy", "slot": "", "effects": [{"type": "stat", "stat": "CONS", "value": 2}]}
        ],
        "items": [
            {"id": "great_staff", "name": "Great Staff", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "staff", "two_handed": true, "base_power": 20},
            {"id": "plain_sword", "name": "Plain Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10},
            {"id": "shaky_ring", "name": "Shaky Ring", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [{"type": "forbid_hands", "hands": 2}, {"type": "pierce_melee"}]}
        ],
        "unit_tracks": tracks
    }))
}

fn growth(offset: i32) -> Growth {
    Growth { max_hp_offset: offset, ..Growth::default() }
}

#[test]
fn the_member_grants_read_and_stay_off_gear() {
    let list = grants::read(&[json!({"type": "forbid_hands", "hands": 2}), json!({"type": "pierce_melee"})]);
    assert_eq!(list, vec![Grant::ForbidHands { hands: 2 }, Grant::PierceMelee]);
    assert!(list.iter().all(|g| !g.gear_ok() && !g.band_ok()), "passives only");
}

#[test]
fn a_passive_applies_them_and_a_worn_item_does_not() {
    let cat = catalog(json!([]));
    let row = &cat.units["adept"];
    let plain = build_from(&cat, UnitBuild::row(row, 0, None)).unit;
    assert_eq!((plain.forbid_hands, plain.pierce_melee), (0, false));
    let over = Overrides { passives: vec!["shakes".into()], ..Overrides::default() };
    let shaken = build_from(&cat, UnitBuild { over, ..UnitBuild::row(row, 0, None) }).unit;
    assert_eq!((shaken.forbid_hands, shaken.pierce_melee), (2, true));
    assert!(shaken.forbids(&cat.items["great_staff"]) && !shaken.forbids(&cat.items["plain_sword"]));
    let ring = Growth { equipment: vec![(Slot::Accessory1, cat.items["shaky_ring"].clone())], ..Growth::default() };
    let ringed = build_from(&cat, UnitBuild { growth: Some(ring), ..UnitBuild::row(row, 0, None) }).unit;
    assert!(ringed.inventory.get(Slot::Accessory1).is_some(), "worn");
    assert_eq!((ringed.forbid_hands, ringed.pierce_melee), (0, false), "gear carries no member grant");
}

#[test]
fn forbid_hands_refuses_exactly_two_handed_items() {
    let cat = catalog(json!([]));
    for (hands, staff, sword) in [(0, false, false), (2, true, false)] {
        assert_eq!(forbids_hands(hands, &cat.items["great_staff"]), staff, "hands {hands}: the two-handed staff");
        assert_eq!(forbids_hands(hands, &cat.items["plain_sword"]), sword, "hands {hands}: the one-handed sword");
    }
}

/// The offset adds to the derived max on every sync (a re-sync keeps it once); a large negative one floors at 1 HP.
#[test]
fn the_max_hp_offset_rides_every_sync() {
    let cat = catalog(json!([]));
    let row = &cat.units["adept"];
    let base = build_from(&cat, UnitBuild { growth: Some(growth(0)), ..UnitBuild::row(row, 0, None) }).unit.max_hp;
    let mut up = build_from(&cat, UnitBuild { growth: Some(growth(OFFSET)), ..UnitBuild::row(row, 0, None) }).unit;
    assert_eq!((up.max_hp, up.hp, up.max_hp_offset), (base + OFFSET, base + OFFSET, OFFSET));
    up.sync_from_stats_and_gear(&cat);
    assert_eq!(up.max_hp, base + OFFSET, "not stacked");
    let down = build_from(&cat, UnitBuild { growth: Some(growth(-OFFSET)), ..UnitBuild::row(row, 0, None) }).unit;
    assert_eq!(down.max_hp, base - OFFSET);
    let floor = build_from(&cat, UnitBuild { growth: Some(growth(-10 * base)), ..UnitBuild::row(row, 0, None) }).unit;
    assert_eq!(floor.max_hp, 1, "at least 1");
    let npc = build_from(&cat, UnitBuild::row(row, 0, None)).unit;
    assert_eq!(npc.max_hp, base, "a unit without growth has no offset");
}

/// A growth unit holds its tracks only while the catalog has a track row; a catalog unit never.
#[test]
fn a_member_holds_tracks_once_a_track_row_exists() {
    let tracks = json!([{"id": "t_track", "name": "Track", "mark": "t_mark", "cause": "t_cause", "stages": [{"name": "One"}]}]);
    let mut stages = IndexMap::new();
    stages.insert("t_track".to_string(), 1);
    let g = Growth { tracks: stages.clone(), ..Growth::default() };
    for (cat, want) in [(catalog(json!([])), None), (catalog(tracks), Some(stages))] {
        let row = &cat.units["adept"];
        let member = build_from(&cat, UnitBuild { growth: Some(g.clone()), ..UnitBuild::row(row, 0, None) }).unit;
        assert_eq!(member.tracks, want);
        assert_eq!(build_from(&cat, UnitBuild::row(row, 0, None)).unit.tracks, None, "a catalog unit holds none");
    }
}

/// A passive granted mid-battle applies once and keeps the battle's max HP and HP (a CONS bonus does not heal or
/// raise the max mid-fight).
#[test]
fn a_passive_granted_now_keeps_the_battle_hp() {
    let cat = catalog(json!([]));
    let mut u = build_from(&cat, UnitBuild { growth: Some(growth(0)), ..UnitBuild::row(&cat.units["adept"], 0, None) }).unit;
    u.max_hp -= 7;
    u.hp = u.max_hp - 3;
    let (max, hp) = (u.max_hp, u.hp);
    assert!(u.grant_passive_now(&cat, &cat.passives["sturdy"]));
    assert_eq!((u.max_hp, u.hp), (max, hp));
    assert!(u.passives.contains(&"sturdy".to_string()));
    assert!(!u.grant_passive_now(&cat, &cat.passives["sturdy"]), "once");
}
