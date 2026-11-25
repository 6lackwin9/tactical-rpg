//! R4-1 `U2.3-TIERS` slice 2 (FB-144) on fixture rows (no shipped data): a member's automatic spend
//! (`progression::auto_for`) reads its base unit through its role template (`rpgkit_rules::unit::spend_growth`,
//! `template_for`): a party base unit never takes one (the class's growth shares), a non-party, non-hero base unit
//! spends by its template's growth shares.

use indexmap::IndexMap;
use rpgkit_rules::stats::{auto_spend, spend_pool};
use rpgkit_world::state::UnitState;
use rpgkit_world::{progression, Data};
use serde_json::json;

fn data() -> Data {
    let base = json!({"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "ai_role": "AGGRESSOR", "base_stats": base, "growth": {"CONS": 2, "STR": 2, "PERC": 1}}],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
            {"id": "grunt", "name": "Grunt", "side": "enemy", "class_id": "drill", "level": 1, "tier": "minion"}
        ],
        "unit_templates": [{"id": "quick", "role": "AGGRESSOR",
            "base_shares": {"STR": 0.15, "DEX": 0.25, "INT": 0.1, "CONS": 0.2, "PERC": 0.15, "KNOW": 0.15},
            "growth_shares": {"DEX": 0.6, "CONS": 0.4}}]
    });
    Data::from_values(&catalog, &json!({}), &json!({}))
}

fn auto(d: &Data, base_unit: &str) -> IndexMap<String, i64> {
    let mut m = UnitState { id: base_unit.into(), base_unit: base_unit.into(), ..UnitState::default() };
    progression::auto_for(d, &mut m);
    m.spend.into_iter().filter(|(_, v)| *v > 0).collect()
}

fn expected(growth: &IndexMap<String, f64>, d: &Data) -> IndexMap<String, i64> {
    let pool = spend_pool(UnitState::default().level(d) as i32);
    auto_spend(growth, pool).into_iter().filter(|(_, v)| *v > 0).map(|(k, v)| (k, v as i64)).collect()
}

#[test]
fn the_auto_spend_reads_the_role_template() {
    let d = data();
    let class_growth = d.cat.classes["drill"].unit_shares().growth;
    let template_growth = d.cat.unit_templates["AGGRESSOR"].shares.growth.clone();
    assert_ne!(expected(&class_growth, &d), expected(&template_growth, &d), "the fixture tells the two apart");
    assert_eq!(auto(&d, "hero"), expected(&class_growth, &d), "a party base unit: the class's growth");
    assert_eq!(auto(&d, "grunt"), expected(&template_growth, &d), "a templated base unit: the template's growth");
}
