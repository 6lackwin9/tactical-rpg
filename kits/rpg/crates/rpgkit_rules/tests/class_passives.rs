//! CLASSPASS1 (`PR5-62a`) on a scratch catalog built here (no shipped data):
//! - Hook 1: `ClassDef::passives` applies slotless class passives to derived units under `DeckRule` before deck
//!   passives (both static catalog units and `Growth` units), deduplicated, and does nothing in classic or fixed units.
//! - Hook 2: `Unit::has_dual_wield` and `when`-gated passive grants (`when: {"dual_wield": "1h_light"}` on `crit`,
//!   `skill_potency`, `stat`, `skill_reach`) update cleanly across `sync_from_stats_and_gear` without stale leaks.
//! - Hook 3: unconditional `"crit"` passive grant adds to `Unit::passive_crit` and `damage::crit_chance`.
//! - Hook 4: `"channel"` group matches `skill.channel.only` skills in `skill_in_group` and `potency_for_skill`.
//! - Hook 5: `SkillDef::always_ranged` + `SkillReach::ranged` + `skill_is_ranged` across `always_ranged`, melee,
//!   `reach_max > 1`, `position` dash-attacks, and user-centred areas.
//! - Hook 6: `STAT_OVERRIDE_FACTOR` (`0.75`) gates off-hand `power_stat_override` and scales overridden unit power.

use rpgkit_data::{Catalog, SkillReach, STAT_OVERRIDE_FACTOR};
use rpgkit_rules::damage::{attack_power, crit_chance};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::ruleset::DeckSlots;
use rpgkit_rules::unit::{skill_in_group, skill_is_ranged, skill_reach_matches};
use rpgkit_rules::{build_from, build_unit, build_unit_in, DeckRule, Growth, UnitBuild};
use serde_json::json;

fn deck_rule(passive_slots: i32) -> DeckRule {
    DeckRule {
        basics: vec!["sk_strike".into()],
        slots: DeckSlots { active: 2, passive: passive_slots, keystone: 0 },
    }
}

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 12, "INT": 12, "CONS": 10, "PERC": 10, "KNOW": 6});
    let growth = json!({"STR": 0, "DEX": 0, "INT": 0, "CONS": 1, "PERC": 0, "KNOW": 0});
    Catalog::from_value(&json!({
        "item_families": [
            {"id": "dagger", "name": "Dagger", "item_type": "WEAPON", "slot_rule": "1h_light"},
            {"id": "mace", "name": "Mace", "item_type": "WEAPON", "slot_rule": "1h_heavy"},
            {"id": "bow", "name": "Bow", "item_type": "WEAPON", "slot_rule": "2h_rig", "rules": ["always_ranged"]},
            {"id": "orb", "name": "Orb", "item_type": "WEAPON", "slot_rule": "off_only", "power_stat_override": "INT"},
            {"id": "shield", "name": "Shield", "item_type": "SHIELD", "slot_rule": "off_only"}
        ],
        "skills": [
            {"id": "sk_strike", "name": "Strike", "kind": "ATTACK", "level": 1, "basic": true, "power_mult": 1.0, "target_mode": "ENEMY"},
            {"id": "sk_slash", "name": "Slash", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"},
            {"id": "sk_shot", "name": "Shot", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["bow"]},
            {"id": "sk_mixed", "name": "Mixed", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "weapon_types": ["bow", "dagger"]},
            {"id": "sk_volley", "name": "Volley", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "reach_max": 3},
            {"id": "sk_dash", "name": "Dash", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "reach_max": 2, "position": "flank"},
            {"id": "sk_sweep", "name": "Sweep", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "reach_max": 2, "area": "ARC"},
            {"id": "sk_chan", "name": "Channel Burst", "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY", "channel": {"only": true}}
        ],
        "passives": [
            {"id": "p_class_crit", "name": "Class Edge", "slot": "", "effects": [
                {"type": "crit", "value": 7.0},
                {"type": "stat", "stat": "DEX", "value": 2},
                {"type": "skill_potency", "group": "channel", "value": 0.25}
            ]},
            {"id": "p_dw", "name": "Twin Tempo", "slot": "", "effects": [
                {"type": "crit", "value": 8.0, "when": {"dual_wield": "1h_light"}},
                {"type": "skill_potency", "family": "sk_slash", "value": 0.15, "when": {"dual_wield": "1h_light"}},
                {"type": "stat", "stat": "STR", "value": 3, "when": {"dual_wield": "1h_light"}},
                {"type": "skill_reach", "reach": 1, "ranged": false, "when": {"dual_wield": "1h_light"}}
            ]},
            {"id": "p_slot", "name": "Slotted", "slot": "passive", "effects": [{"type": "crit", "value": 4.0}]}
        ],
        "classes": [
            {
                "id": "c_derived", "name": "Derived",
                "base_stats": stats, "growth": growth,
                "proficiency": {"dagger": {"level": 2, "scaling": 1.0}, "mace": {"level": 2, "scaling": 1.0}, "orb": {"level": 2, "scaling": 1.0}, "shield": {"level": 2}},
                "skill_path": {"sk_slash": 1, "sk_chan": 1},
                "passives": ["p_class_crit", "p_dw", "p_class_crit"],
                "main_hand": "w_dagger_a", "off_hand": "w_dagger_b"
            }
        ],
        "items": [
            {"id": "w_dagger_a", "name": "Dagger A", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "dagger", "family_slot_rule": "1h_light", "base_power": 10},
            {"id": "w_dagger_b", "name": "Dagger B", "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": "dagger", "family_slot_rule": "1h_light", "base_power": 8},
            {"id": "w_mace", "name": "Mace", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "mace", "family_slot_rule": "1h_heavy", "base_power": 10},
            {"id": "w_mace_off", "name": "Mace Off", "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": "mace", "family_slot_rule": "1h_heavy", "base_power": 8},
            {"id": "w_orb", "name": "Orb", "item_type": "WEAPON", "slot": "OFF_HAND", "weapon_type": "orb", "family_slot_rule": "off_only", "family_power_stat_override": "INT", "base_power": 2},
            {"id": "w_shield", "name": "Shield", "item_type": "SHIELD", "slot": "OFF_HAND", "weapon_type": "shield", "family_slot_rule": "off_only", "mitigation": 4}
        ],
        "units": [
            {"id": "u_derived", "name": "Derived", "side": "party", "class_id": "c_derived", "level": 1},
            {"id": "u_fixed", "name": "Fixed", "side": "enemy", "stat_mode": "fixed", "class_id": "c_derived", "level": 1, "hp": 40, "speed": 10, "attack_range": 1,
             "deck": {"passive": ["p_slot"]}}
        ]
    }))
}

#[test]
fn class_passives_apply_under_deck_only_and_on_growth_units() {
    let cat = catalog();
    let rule = deck_rule(0);

    // Classic build ignores `class.passives` (and `p_dw`), keeping baseline stats/crit.
    let classic = build_unit(&cat, &cat.units["u_derived"], 0);
    assert!(classic.passives.is_empty(), "classic gets no class passives: {:?}", classic.passives);
    assert_eq!(classic.passive_crit, 0.0);
    assert_eq!(classic.cond.crit, 0.0);

    // Derived unit under `DeckRule` (even with 0 passive deck slots) gets slotless `class.passives` once.
    let deck_u = build_unit_in(&cat, &cat.units["u_derived"], 0, Some(&rule));
    assert_eq!(deck_u.passives, ["p_class_crit", "p_dw"]);
    assert_eq!(deck_u.passive_crit, 7.0);
    assert_eq!(deck_u.stats.stat("DEX", None), 14, "p_class_crit +2 DEX");

    // Derived growth unit (party member) under `DeckRule` also receives `class.passives`.
    let member = build_from(&cat, UnitBuild { growth: Some(Growth::default()), ..UnitBuild::row(&cat.units["u_derived"], 0, Some(&rule)) }).unit;
    assert_eq!(member.passives, ["p_class_crit", "p_dw"]);
    assert_eq!(member.passive_crit, 7.0);

    // Fixed-stat unit receives no `class.passives` even under `DeckRule` (only its deck passive `p_slot`).
    let fixed_rule = deck_rule(1);
    let fixed = build_unit_in(&cat, &cat.units["u_fixed"], 0, Some(&fixed_rule));
    assert_eq!(fixed.passives, ["p_slot"]);
    assert_eq!(fixed.passive_crit, 4.0);
}

#[test]
fn conditional_dual_wield_grants_track_equipped_gear_cleanly() {
    let cat = catalog();
    let rule = deck_rule(0);
    let mut u = build_unit_in(&cat, &cat.units["u_derived"], 0, Some(&rule));

    assert!(u.has_dual_wield("1h_light"));
    assert!(!u.has_dual_wield("1h_heavy"));
    assert!(!u.has_dual_wield(""));
    assert_eq!(u.passive_crit, 7.0);
    assert_eq!(u.cond.crit, 8.0);
    // Base crit 6.0 (PERC 10) + passive 7.0 + conditional 8.0 = 21.0.
    assert!((crit_chance(&u) - 21.0).abs() < 1e-9, "got {}", crit_chance(&u));
    assert!((u.potency_for("sk_slash") - 1.15).abs() < 1e-9);
    assert_eq!(u.potency_parts(&cat.skills[1]).len(), 1);
    assert_eq!(u.stats.stat("STR", None), 15, "base 12 + conditional 3");
    assert_eq!(u.skill_reach_bonus(&cat.skills[1]), 1, "conditional melee reach +1");

    // Re-syncing keeps values identical (no stacking).
    u.sync_from_stats_and_gear(&cat);
    assert_eq!(u.cond.crit, 8.0);
    assert_eq!(u.stats.stat("STR", None), 15);

    // Replacing off-hand dagger with shield turns off `dual_wield: "1h_light"`.
    u.inventory.equip(Equipment::new(&cat.items["w_shield"]), Slot::OffHand);
    u.sync_from_stats_and_gear(&cat);
    assert!(!u.has_dual_wield("1h_light"));
    assert_eq!(u.cond.crit, 0.0);
    assert!((crit_chance(&u) - 13.0).abs() < 1e-9, "got {}", crit_chance(&u));
    assert!((u.potency_for("sk_slash") - 1.0).abs() < 1e-9);
    assert_eq!(u.potency_parts(&cat.skills[1]).len(), 0);
    assert_eq!(u.stats.stat("STR", None), 12);
    assert_eq!(u.skill_reach_bonus(&cat.skills[1]), 0);

    // Wielding mixed 1h_heavy + 1h_light is not dual_wield for either.
    u.inventory.equip(Equipment::new(&cat.items["w_mace"]), Slot::MainHand);
    u.inventory.equip(Equipment::new(&cat.items["w_dagger_b"]), Slot::OffHand);
    u.sync_from_stats_and_gear(&cat);
    assert!(!u.has_dual_wield("1h_light"));
    assert!(!u.has_dual_wield("1h_heavy"));

    // Dual-wielding two 1h_light weapons again restores conditional grants cleanly.
    u.inventory.equip(Equipment::new(&cat.items["w_dagger_a"]), Slot::MainHand);
    u.inventory.equip(Equipment::new(&cat.items["w_dagger_b"]), Slot::OffHand);
    u.sync_from_stats_and_gear(&cat);
    assert!(u.has_dual_wield("1h_light"));
    assert_eq!(u.cond.crit, 8.0);
    assert_eq!(u.stats.stat("STR", None), 15);
}

#[test]
fn channel_group_and_ranged_reach_filters_match_spec() {
    let cat = catalog();
    let rule = deck_rule(0);
    let u = build_unit_in(&cat, &cat.units["u_derived"], 0, Some(&rule));

    let sk = |id: &str| cat.skill(id).unwrap();
    assert!(skill_in_group(sk("sk_chan"), "channel"));
    assert!(!skill_in_group(sk("sk_slash"), "channel"));
    assert!((u.potency_for_skill(sk("sk_chan")) - 1.25).abs() < 1e-9);
    assert_eq!(u.potency_parts(sk("sk_chan")).len(), 1);

    // `always_ranged` propagation + `skill_is_ranged`.
    assert!(sk("sk_shot").always_ranged);
    assert!(!sk("sk_mixed").always_ranged);
    assert!(!sk("sk_slash").always_ranged);

    assert!(skill_is_ranged(sk("sk_shot")));
    assert!(skill_is_ranged(sk("sk_volley")));
    assert!(!skill_is_ranged(sk("sk_slash")));
    assert!(!skill_is_ranged(sk("sk_mixed")));
    assert!(!skill_is_ranged(sk("sk_dash")));
    assert!(!skill_is_ranged(sk("sk_sweep")));

    let ranged_only = SkillReach { reach: 1, intent: String::new(), kind: String::new(), ranged: Some(true) };
    let melee_only = SkillReach { reach: 1, intent: String::new(), kind: String::new(), ranged: Some(false) };
    let any = SkillReach { reach: 1, intent: String::new(), kind: String::new(), ranged: None };

    for id in ["sk_shot", "sk_volley"] {
        assert!(skill_reach_matches(&ranged_only, sk(id)), "{id}");
        assert!(!skill_reach_matches(&melee_only, sk(id)), "{id}");
        assert!(skill_reach_matches(&any, sk(id)), "{id}");
    }
    for id in ["sk_slash", "sk_mixed", "sk_dash", "sk_sweep"] {
        assert!(!skill_reach_matches(&ranged_only, sk(id)), "{id}");
        assert!(skill_reach_matches(&melee_only, sk(id)), "{id}");
        assert!(skill_reach_matches(&any, sk(id)), "{id}");
    }
}

#[test]
fn stat_override_applies_factor_to_threshold_and_unit_power() {
    let cat = catalog();
    let mut u = build_unit(&cat, &cat.units["u_derived"], 0);
    u.inventory.equip(Equipment::new(&cat.items["w_mace"]), Slot::MainHand);
    u.inventory.equip(Equipment::new(&cat.items["w_orb"]), Slot::OffHand);
    u.sync_from_stats_and_gear(&cat);

    // Equal INT (12) vs STR (12): 12 * 0.75 = 9 <= 12 -> no override.
    let ap_eq = attack_power(&u, false, false);
    assert!(!ap_eq.stat_override);
    assert_eq!(ap_eq.stat_override_factor, 1.0);
    assert_eq!(ap_eq.stat_name, "STR");

    // INT 16 vs STR 12: 16 * 0.75 = 12 == 12 (strict `>` fails) -> no override.
    u.stats.core.insert("INT".into(), 16);
    u.sync_from_stats_and_gear(&cat);
    let ap_tie = attack_power(&u, false, false);
    assert!(!ap_tie.stat_override);
    assert_eq!(ap_tie.stat_name, "STR");

    // INT 20 vs STR 12: 20 * 0.75 = 15.0 > 12 -> override triggers with factor 0.75.
    u.stats.core.insert("INT".into(), 20);
    u.sync_from_stats_and_gear(&cat);
    let ap_over = attack_power(&u, false, false);
    assert!(ap_over.stat_override);
    assert_eq!(ap_over.stat_name, "INT");
    assert_eq!(ap_over.stat_val, 20);
    assert!((ap_over.stat_override_factor - STAT_OVERRIDE_FACTOR).abs() < 1e-9);
    assert!((ap_over.unit_power - 20.0 * 1.0 * STAT_OVERRIDE_FACTOR).abs() < 1e-9);
    assert_eq!(ap_eq.attack_power, 24);
    assert_eq!(ap_over.attack_power, 27, "rises by +3 from unit_power 12 -> 15");
}
