//! U1.2-S4 (U2_CLASS.md feasibility 2: "keep the AI skill cap on tier") on a scratch catalog: a class unit without its
//! own `skill_cap` takes its tier's cap from the kit's tier table (minion 2, as before), and no cap (`NO_CEILING`)
//! where the tier has none; a unit's own cap wins.

use rpgkit_battle::world::{AiProfile, NO_CEILING};
use rpgkit_data::Catalog;
use rpgkit_rules::stats::TIERS;
use serde_json::json;

#[test]
fn the_ai_skill_cap_stays_on_tier() {
    let units: Vec<_> = TIERS.iter().map(|r| json!({"id": format!("u_{}", r.id), "name": r.id, "class_id": "drill", "tier": r.id, "side": "enemy"})).collect();
    let mut rows = units.clone();
    rows.push(json!({"id": "u_own", "name": "own", "class_id": "drill", "tier": "minion", "skill_cap": 4, "side": "enemy"}));
    let cat = Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10}, "growth": {}}],
        "units": rows
    }));
    for r in TIERS.iter() {
        let cap = AiProfile::for_unit(&cat, &cat.units[&format!("u_{}", r.id)]).skill_cap;
        let want = if r.id == "minion" { 2 } else { NO_CEILING };
        assert_eq!(cap, want, "{}", r.id);
        assert_eq!(cap, if r.skill_cap > 0 { r.skill_cap } else { NO_CEILING }, "{}: the table's cap", r.id);
    }
    assert_eq!(AiProfile::for_unit(&cat, &cat.units["u_own"]).skill_cap, 4, "the unit's own cap wins");
}
