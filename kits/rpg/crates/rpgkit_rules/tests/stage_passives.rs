//! PK2-STAGE-PASSIVES (`PR6-2`) rules-layer unit tests on scratch data (no shipped game data):
//! - `Target::Armor` (`stat: "armor"`): passive, gear, conditional, and band grants sum into `base_armor_rating`
//!   alongside gear & `mitigation_rating`, clamped `>= 0`.
//! - `Target::MoveRate` (`stat: "move_rate"`): passive, gear, conditional, and band grants sum into
//!   `Unit::move_per_block` alongside haste/slow, clamped `>= 1`, while `rating_set(MoveRate)` still overrides.
//! - `"ranged"` group in `unit::GROUPS` and `skill_in_group(s, "ranged") == skill_is_ranged(s)`.
//! - `skill_potency` optional `min_distance` (`0..=12`, default `0`): ignored when distance `< min_distance` or `0`,
//!   applied when distance `>= min_distance` across `Unit::potency_for_skill_at` / `power_for_at` and
//!   `damage::compute` / `compute_traced`; a distance-gated `ranged` entry also takes a ranged hit of a skill that is
//!   not itself ranged (the basic Strike with a bow, `PR7-2` N1).

use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, base_armor_rating, compute, compute_traced, compute_traced_skill, HitInput};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::grants::{self, Grant, Modifier, SkillSet, Target};
use rpgkit_rules::ruleset::DeckSlots;
use rpgkit_rules::status::{self, Rating, RatingSet};
use rpgkit_rules::unit::{skill_in_group, skill_is_ranged, GROUPS};
use rpgkit_rules::{build_unit_in, DeckRule, Unit};
use serde_json::json;

fn deck_rule() -> DeckRule {
    DeckRule {
        basics: vec!["sk_strike".into()],
        slots: DeckSlots { active: 2, passive: 2, keystone: 0 },
    }
}

fn catalog() -> Catalog {
    Catalog::from_value(&json!({
        "item_families": [
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot_rule": "1h_heavy"},
            {"id": "bow", "name": "Bow", "item_type": "WEAPON", "slot_rule": "2h_rig", "rules": ["always_ranged"]},
            {"id": "armor", "name": "Armor", "item_type": "ARMOR"}
        ],
        "skills": [
            {"id": "sk_strike", "name": "Strike", "kind": "ATTACK", "level": 1, "basic": true, "power_mult": 1.0, "target_mode": "ENEMY"},
            {"id": "sk_shot", "name": "Shot", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["bow"]},
            {"id": "sk_dash", "name": "Dash", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "reach_max": 2, "position": "flank"},
            {"id": "sk_arc", "name": "Arc", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "reach_max": 2, "area": "ARC"}
        ],
        "passives": [
            {"id": "p_armor_plus", "name": "Armor Plus", "slot": "", "effects": [
                {"type": "stat", "stat": "armor", "value": 2},
                {"type": "stat", "stat": "move_rate", "value": -1}
            ]},
            {"id": "p_armor_drop", "name": "Armor Drop", "slot": "", "effects": [
                {"type": "stat", "stat": "armor", "value": -100},
                {"type": "stat", "stat": "move_rate", "value": -10}
            ]},
            {"id": "p_ranged_far", "name": "Far Penalty", "slot": "", "effects": [
                {"type": "skill_potency", "group": "ranged", "min_distance": 4, "value": -0.15}
            ]},
            {"id": "p_ranged_flat", "name": "Ranged Boost", "slot": "", "effects": [
                {"type": "skill_potency", "group": "ranged", "value": 0.25}
            ]},
            {"id": "p_cond_far", "name": "Conditional Far", "slot": "", "effects": [
                {"type": "skill_potency", "group": "ranged", "min_distance": 4, "value": -0.10, "when": {"ranged": true}},
                {"type": "stat", "stat": "armor", "value": 1, "when": {"ranged": true}},
                {"type": "stat", "stat": "move_rate", "value": 1, "when": {"ranged": true}}
            ]}
        ],
        "items": [
            {"id": "w_sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10},
            {"id": "w_bow", "name": "Bow", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "bow", "base_power": 20, "base_range": 4, "two_handed": true},
            {"id": "a_plate", "name": "Plate", "item_type": "ARMOR", "slot": "BODY", "mitigation": 4},
            {"id": "a_charm", "name": "Charm", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "effects": [
                {"type": "stat", "stat": "armor", "value": 3},
                {"type": "stat", "stat": "move_rate", "value": 1}
            ]}
        ],
        "classes": [
            {
                "id": "c_archer", "name": "Archer",
                "base_stats": {"STR": 10, "DEX": 12, "INT": 8, "CONS": 12, "PERC": 10, "KNOW": 10},
                "growth": {"STR": 0, "DEX": 0, "INT": 0, "CONS": 1, "PERC": 0, "KNOW": 0},
                "proficiency": {"sword": {"level": 1, "scaling": 1.0}, "bow": {"level": 1, "scaling": 1.0}, "armor": {"level": 1}},
                "skill_path": {"sk_shot": 1},
                "main_hand": "w_bow", "body": "a_plate"
            }
        ],
        "units": [
            {"id": "u_hero", "name": "Hero", "side": "party", "class_id": "c_archer", "level": 1},
            {"id": "u_foe", "name": "Foe", "side": "enemy", "stat_mode": "fixed", "hp": 200, "power": 10, "armor": 0, "speed": 10, "attack_range": 1}
        ]
    }))
}

fn hero(cat: &Catalog) -> Unit {
    build_unit_in(cat, &cat.units["u_hero"], 0, Some(&deck_rule()))
}

#[test]
fn target_armor_and_move_rate_parse_and_sum_with_floors() {
    let parsed = grants::read(&[
        json!({"type": "stat", "stat": "armor", "value": 2}),
        json!({"type": "stat", "stat": "move_rate", "value": -1}),
        json!({"type": "skill_potency", "group": "ranged", "min_distance": 4, "value": -0.15}),
    ]);
    assert_eq!(
        parsed,
        vec![
            Grant::Modifier(Modifier { target: Target::Armor, value: 2.0 }),
            Grant::Modifier(Modifier { target: Target::MoveRate, value: -1.0 }),
            Grant::Modifier(Modifier { target: Target::SkillPotency { set: SkillSet::Group("ranged".into()), min_distance: 4 }, value: -0.15 }),
        ]
    );

    let cat = catalog();
    let mut u = hero(&cat);
    let base_arm = base_armor_rating(&u);
    let base_mv = u.move_per_block();
    assert!(base_mv >= 2, "fixture hero needs >=2 move rate so -1 drops it cleanly");

    u.apply_passive(&cat, &cat.passives["p_armor_plus"]);
    assert_eq!(base_armor_rating(&u), base_arm + 2);
    assert_eq!(u.move_per_block(), (base_mv - 1).max(1));

    // Haste & slow still combine on top of passive_move_rate; rating_set still overrides cleanly.
    status::grant_now(&mut u, "haste", 1, 1, "test");
    assert_eq!(u.move_per_block(), base_mv);
    status::remove_active(&mut u, "haste");
    u.rating_sets.push(RatingSet { rating: Rating::MoveRate, value: 5, rounds: 1 });
    assert_eq!(u.move_per_block(), 5);
    u.rating_sets.clear();

    // Worn gear stat: armor / move_rate clears cleanly across sync_from_stats_and_gear.
    u.inventory.equip(Equipment::new(&cat.items["a_charm"]), Slot::Accessory1);
    u.sync_from_stats_and_gear(&cat);
    assert_eq!(base_armor_rating(&u), base_arm + 2 + 3);
    assert_eq!(u.move_per_block(), base_mv);
    u.inventory.unequip(Slot::Accessory1);
    u.sync_from_stats_and_gear(&cat);
    assert_eq!(base_armor_rating(&u), base_arm + 2);
    assert_eq!(u.move_per_block(), (base_mv - 1).max(1));

    // Meter bands apply and remove armor/move_rate/distance_potency cleanly.
    let mut b = hero(&cat);
    let applied = b.apply_band(&cat, &cat.passives["p_armor_plus"].effects, "meter:m");
    assert_eq!(base_armor_rating(&b), base_arm + 2);
    assert_eq!(b.move_per_block(), (base_mv - 1).max(1));
    b.remove_band(&applied);
    assert_eq!(base_armor_rating(&b), base_arm);
    assert_eq!(b.move_per_block(), base_mv);

    // Extreme negative armor/move_rate clamps to >= 0 armor and >= 1 move_per_block.
    let mut floored = hero(&cat);
    floored.apply_passive(&cat, &cat.passives["p_armor_drop"]);
    assert_eq!(base_armor_rating(&floored), 0);
    assert_eq!(damage::armor_rating(&floored), 0);
    assert_eq!(floored.move_per_block(), 1);
}

#[test]
fn ranged_group_and_min_distance_evaluate_cleanly() {
    let cat = catalog();
    assert!(GROUPS.contains(&"ranged"));
    for id in ["sk_shot", "sk_strike", "sk_dash", "sk_arc"] {
        let sk = cat.skill(id).unwrap();
        assert_eq!(skill_in_group(sk, "ranged"), skill_is_ranged(sk), "{id}");
    }

    let mut u = hero(&cat);
    let target = build_unit_in(&cat, &cat.units["u_foe"], 0, None);
    let shot = cat.skill("sk_shot").unwrap();
    let strike = cat.skill("sk_strike").unwrap();

    u.apply_passive(&cat, &cat.passives["p_ranged_far"]);
    // Distance 0 / 3 do not trigger `min_distance: 4`.
    assert!((u.potency_for_skill(shot) - 1.0).abs() < 1e-9);
    assert!((u.potency_for_skill_at(shot, true, 3) - 1.0).abs() < 1e-9);
    assert!((u.potency_for_skill_at(shot, true, 4) - 0.85).abs() < 1e-9);
    assert!((u.potency_for_skill_at(shot, true, 6) - 0.85).abs() < 1e-9);
    // PR7-2 (N1): the basic Strike is not itself a ranged skill, but a ranged hit of it (Strike with the bow) takes the
    // distance-gated `group: "ranged"` grant beyond 3 tiles, by the hit's ranged flag; not at <= 3, never a melee hit.
    assert!(!skill_is_ranged(strike));
    assert!((u.potency_for_skill_at(strike, true, 3) - 1.0).abs() < 1e-9);
    assert!((u.potency_for_skill_at(strike, true, 4) - 0.85).abs() < 1e-9);
    assert!((u.power_for_at(strike, true, 4) - 0.85 * u.power_for(strike)).abs() < 1e-9);
    assert_eq!(u.potency_parts_at(strike, true, 4).len(), 1);
    assert!((u.potency_for_skill_at(strike, false, 4) - 1.0).abs() < 1e-9);
    assert!(u.potency_parts_at(strike, false, 4).is_empty());

    let base_dmg = compute(&u, &target, HitInput { ranged: true, distance: 3, ..HitInput::default() }).final_dmg;
    let close_dmg = compute(&u, &target, HitInput { ranged: true, distance: 0, ..HitInput::default() }).final_dmg;
    let far_hit = compute(&u, &target, HitInput { ranged: true, distance: 4, ..HitInput::default() });
    let far_traced = compute_traced(&u, &target, HitInput { ranged: true, distance: 4, ..HitInput::default() });
    let far_traced_skill = compute_traced_skill(&u, &target, HitInput { ranged: true, distance: 4, ..HitInput::default() }, shot);
    let melee_close = compute(&u, &target, HitInput { ranged: false, distance: 3, ..HitInput::default() }).final_dmg;
    let melee_far = compute(&u, &target, HitInput { ranged: false, distance: 4, ..HitInput::default() }).final_dmg;
    assert_eq!(close_dmg, base_dmg);
    assert_eq!(melee_far, melee_close);
    assert!(far_hit.final_dmg < base_dmg, "far {} should be < base {}", far_hit.final_dmg, base_dmg);
    assert_eq!(far_traced.final_dmg, far_hit.final_dmg);
    assert_eq!(far_traced_skill.final_dmg, far_hit.final_dmg);
    let dist_step = far_traced.trace.step("distance_potency").expect("distance_potency step in trace");
    assert_eq!(dist_step.parts.len(), 1);
    assert_eq!(dist_step.parts[0].source.as_ref(), "passive:p_ranged_far");

    // Distance-gated potency adds linearly alongside distance-0 ranged potency, and `when` updates cleanly on gear swap.
    u.apply_passive(&cat, &cat.passives["p_ranged_flat"]);
    assert!((u.potency_for_skill_at(shot, true, 3) - 1.25).abs() < 1e-9);
    assert!((u.potency_for_skill_at(shot, true, 4) - 1.10).abs() < 1e-9);
    assert_eq!(u.potency_parts_at(shot, true, 3).len(), 1);
    assert_eq!(u.potency_parts_at(shot, true, 4).len(), 2);
    // N1 reaches only the distance-gated entries: the distance-0 ranged boost stays with inherently ranged skills.
    assert!((u.potency_for_skill_at(strike, true, 4) - 0.85).abs() < 1e-9);
    assert_eq!(u.potency_parts_at(strike, true, 4).len(), 1);

    u.apply_passive(&cat, &cat.passives["p_cond_far"]);
    // Hero currently wields `w_bow` (`attack_range > 1`), so `when: {"ranged": true}` holds.
    assert!((u.potency_for_skill_at(shot, true, 4) - 1.00).abs() < 1e-9);
    u.inventory.equip(Equipment::new(&cat.items["w_sword"]), Slot::MainHand);
    u.sync_from_stats_and_gear(&cat);
    assert!((u.potency_for_skill_at(shot, true, 4) - 1.10).abs() < 1e-9);
}
