//! CLASS-BASICS (READY row R2-W2b, user 2026-11-08) on a scratch catalog built here (no shipped data): a class's
//! `basics` (skill ids) join the ruleset's basics in the unit's deck rule (`DeckRule::with_class`): after the ruleset's,
//! by family, a repeat once. Under the rule `deck` they are open basics: owned and usable without a deck slot, never an
//! active, for a static unit (an enemy, a Dojo or sim unit) and a growth unit (a party member) alike; only units of that
//! class get them. Classic (no deck rule) is untouched.

use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::DeckSlots;
use rpgkit_rules::{build_from, build_unit, build_unit_in, DeckRule, Growth, UnitBuild};
use serde_json::json;

fn catalog() -> Catalog {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10});
    let class = |id: &str, path: serde_json::Value, basics: serde_json::Value| {
        json!({"id": id, "name": id, "base_stats": stats, "growth": {}, "proficiency": {"sword": {"level": 1}}, "skill_path": path, "basics": basics})
    };
    let attack = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "power_mult": 1.0, "target_mode": "ENEMY"});
    Catalog::from_value(&json!({
        "classes": [
            // `strike` repeats a ruleset basic: it counts once
            class("guardian", json!({"slash": 1, "chop": 1}), json!(["brace", "strike"])),
            class("cutter", json!({"slash": 1}), json!(["flank_cut"])),
            class("plain", json!({"slash": 1}), json!([]))
        ],
        "units": [
            {"id": "guardian", "name": "Guardian", "class_id": "guardian", "level": 1, "side": "enemy"},
            {"id": "cutter", "name": "Cutter", "class_id": "cutter", "level": 1, "side": "enemy"},
            {"id": "plain", "name": "Plain", "class_id": "plain", "level": 1, "side": "player"}
        ],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "basic": true},
            {"id": "brace", "name": "Brace", "kind": "DEFEND", "level": 1, "learned_only": true, "round_intent": "guard", "mitigation": 0.2},
            {"id": "flank_cut", "name": "Flank Cut", "kind": "ATTACK", "level": 1, "round_intent": "maneuver", "power_mult": 0.6},
            attack("slash"),
            attack("chop")
        ]
    }))
}

fn rule(active: i32) -> DeckRule {
    DeckRule { basics: vec!["strike".into()], slots: DeckSlots { active, passive: 0, keystone: 0 } }
}

#[test]
fn class_basics_follow_the_ruleset_basics_once_each() {
    let cat = catalog();
    let base = rule(2);
    assert_eq!(base.with_class(&cat, "guardian").basics, ["strike", "brace"], "the ruleset's first, a repeat once");
    assert_eq!(base.with_class(&cat, "guardian").slots, base.slots, "the slots stay the ruleset's");
    assert_eq!(base.with_class(&cat, "cutter").basics, ["strike", "flank_cut"]);
    assert_eq!(base.with_class(&cat, "plain"), base, "a class without basics: unchanged");
    assert_eq!(base.with_class(&cat, "nobody"), base, "an unknown class: unchanged");
    assert_eq!(base.with_class_of(&cat, "guardian"), base.with_class(&cat, "guardian"), "by the unit row's class");
    let twice = base.with_class(&cat, "guardian");
    assert_eq!(twice.with_class(&cat, "guardian"), twice, "applying it again adds nothing");
}

#[test]
fn a_static_unit_under_the_deck_rule_uses_its_class_basic_without_a_slot() {
    let cat = catalog();
    let mut def = cat.units["guardian"].clone();
    // the deck names the class basic first: it is skipped, so the one active slot still holds `slash`
    def.deck.active = vec!["brace".into(), "slash".into()];
    let u = build_unit_in(&cat, &def, 0, Some(&rule(1)));
    let rule_of = u.deck_rule.as_ref().expect("built under the rule");
    assert!(rule_of.basics.contains(&"brace".to_string()), "the class basic is in the unit's deck rule");
    assert_eq!(u.active_families(), ["slash"], "the class basic takes no active slot");
    let brace = cat.skill("brace").unwrap();
    assert_eq!(u.deck_basic(&cat, "brace"), Some(true), "an open basic");
    assert!(u.owns_skill(&cat, "brace") && u.always_active(&cat, "brace") && u.can_use(&cat, brace), "owned and usable");
}

#[test]
fn an_auto_deck_fills_its_slots_past_the_class_basic() {
    let cat = catalog();
    let u = build_unit_in(&cat, &cat.units["guardian"], 0, Some(&rule(2)));
    assert!(!u.active_families().contains(&"brace".to_string()), "never an active");
    assert!(u.can_use(&cat, cat.skill("brace").unwrap()));
}

#[test]
fn only_units_of_the_class_get_its_basics() {
    let cat = catalog();
    let cutter = build_unit_in(&cat, &cat.units["cutter"], 0, Some(&rule(2)));
    assert!(!cutter.owns_skill(&cat, "brace"), "another class's basic stays closed");
    assert!(cutter.can_use(&cat, cat.skill("flank_cut").unwrap()), "its own class basic is open");
    let plain = build_unit_in(&cat, &cat.units["plain"], 0, Some(&rule(2)));
    assert_eq!(plain.deck_rule, Some(rule(2)), "no class basics: the ruleset's terms");
    assert!(!plain.owns_skill(&cat, "flank_cut") && !plain.owns_skill(&cat, "brace"));
}

#[test]
fn a_growth_unit_gets_its_class_basics_too() {
    let cat = catalog();
    let base = rule(1);
    let built = build_from(&cat, UnitBuild { growth: Some(Growth::default()), ..UnitBuild::row(&cat.units["guardian"], 0, Some(&base)) }).unit;
    assert_eq!(built.deck_rule, Some(base.with_class(&cat, "guardian")));
    assert!(built.can_use(&cat, cat.skill("brace").unwrap()));
}

#[test]
fn classic_ignores_class_basics() {
    let cat = catalog();
    let u = build_unit(&cat, &cat.units["guardian"], 0);
    assert!(u.deck_rule.is_none());
    assert!(!u.owns_skill(&cat, "brace"), "no deck rule: a class basic opens nothing");
}
