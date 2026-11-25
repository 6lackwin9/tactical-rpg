//! A1.2-S1 (docs/sessions/A1_VALUE.md, FB-55): an item use costs the ruleset's `item_ap` (classic 1, `round_intent`
//! rows 0, a row may set 0..2); a unit that pays no AP (an enemy under `enemy_moves`) pays nothing. Expectations come
//! from the ruleset numbers, never from shipped data.

use indexmap::IndexMap;
use rpgkit_battle::command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::PlanStep;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::{Catalog, ItemDef};
use rpgkit_rules::ruleset::{self, Ruleset, ENEMY_MOVES, ROUND_INTENT};
use serde_json::json;
use std::sync::Arc;

const UNIT_AP: i32 = 10;
const MEDIC: UnitId = 0;
const TANK: UnitId = 1;
const FOE: UnitId = 2;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("t_medic", "party"), unit("t_tank", "party"), unit("t_foe", "enemy")]}))
}

fn row(v: serde_json::Value) -> Ruleset {
    Ruleset::from_row(v.as_object().unwrap())
}

/// The medic and the tank side by side, the foe away; two potions in every unit's pouch.
fn session(rules: Ruleset) -> Session {
    let enc = json!({"id": "t_fight", "name": "Fight", "party": ["t_medic", "t_tank"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(enc.as_object().unwrap());
    def.ruleset = rules;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(MEDIC, V2::new(4, 4)), (TANK, V2::new(5, 4)), (FOE, V2::new(11, 8))] {
        s.w.grid.positions[c] = Some(cell);
    }
    let potion = ItemDef::from_obj(json!({"id": "t_potion", "name": "Potion", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 20}).as_object().unwrap());
    for c in [MEDIC, TANK, FOE] {
        for _ in 0..2 {
            assert!(s.w.units[c].inventory.add_to_pouch(&potion));
        }
    }
    s.w.add_item_skills();
    s
}

/// The AP the medic's planned potion on the tank costs.
fn planned_item_ap(rules: Ruleset) -> i32 {
    let mut s = session(rules);
    let problem = s.plan.add_step(&s.w, MEDIC, PlanStep::use_item("t_potion", TANK), true);
    assert_eq!(problem, "", "the potion plans");
    let entries = s.plan.entries_for(&s.w, MEDIC);
    let e = entries.iter().find(|e| e.item.is_some()).expect("the item entry");
    e.total_ap()
}

/// The default is `ITEM_AP` (1) whatever the rules (a `round_intent` row is no gate); a set value wins (the shipped
/// `timeline` row sets 0 in data); the field reads in row form.
#[test]
fn item_ap_reads_by_the_rules_and_the_row() {
    assert_eq!(ruleset::ITEM_AP, 1, "A1.2-S1: classic keeps ITEMUSE1's 1 AP");
    assert_eq!(Ruleset::classic().numbers.item_ap, ruleset::ITEM_AP);
    assert_eq!(row(json!({"id": "t", "rules": []})).numbers.item_ap, ruleset::ITEM_AP);
    assert_eq!(row(json!({"id": "t", "rules": [ROUND_INTENT]})).numbers.item_ap, ruleset::ITEM_AP, "round_intent does not change the default");
    assert_eq!(row(json!({"id": "t", "rules": [ROUND_INTENT], "item_ap": 0})).numbers.item_ap, 0, "the timeline row's value");
    let set = row(json!({"id": "t", "rules": [ROUND_INTENT], "item_ap": 2}));
    assert_eq!(set.numbers.item_ap, 2);
    assert_eq!(set.numbers.field("item_ap").as_deref(), Some("2"));
    assert!(ruleset::PROMOTED.contains(&("ITEM_AP", "item_ap")));
}

/// A1 parameter table: `item_ap` 0..2; a row outside is a content check error.
#[test]
fn item_ap_outside_its_range_is_an_error() {
    assert_eq!(ruleset::field_rule_of("item_ap").and_then(|f| f.range), Some((0.0, 2.0)));
    assert!(row(json!({"id": "t", "rules": [], "item_ap": 2})).notes().errors.is_empty());
    assert_eq!(row(json!({"id": "t", "rules": [], "item_ap": 3})).notes().errors, ["item_ap 3 is outside 0..2."]);
}

/// The planner charges the ruleset's `item_ap` for a pouch item under classic, `round_intent` and a set value.
#[test]
fn a_planned_item_costs_the_rulesets_item_ap() {
    for rules in [Ruleset::classic(), row(json!({"id": "t", "rules": [ROUND_INTENT], "item_ap": 0})), row(json!({"id": "t", "rules": [], "item_ap": 2}))] {
        let want = rules.numbers.item_ap;
        assert_eq!(planned_item_ap(rules.clone()), want, "{:?}", rules.rules);
        let s = session(rules);
        assert_eq!(command::item_ap(&s.w, MEDIC), want);
    }
}

/// RULESET1 T11: an enemy under `enemy_moves` pays no AP, so no item AP either, whatever `item_ap` says.
#[test]
fn a_unit_that_pays_no_ap_pays_no_item_ap() {
    let s = session(row(json!({"id": "t", "rules": [ENEMY_MOVES], "item_ap": 2})));
    assert!(!s.w.pays_ap(FOE));
    assert_eq!(command::item_ap(&s.w, FOE), 0);
    assert_eq!(command::item_ap(&s.w, MEDIC), 2, "the party still pays");
}
