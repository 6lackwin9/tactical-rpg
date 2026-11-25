//! RULESET1-V through the host on a scratch catalog: under the rule `plan_badges` the `intents` reply gains
//! `round_intents` (one record per active unit, both sides) and `intel {unit}` gives one unit's record; `new`
//! takes `plan_reveal`; a classic battle keeps the `intents` reply's shape and refuses `intel`.

use rpgkit_data::Catalog;
use rpgkit_host::{Battle, Host};
use serde_json::{json, Value};
use std::sync::Arc;

fn catalog() -> Arc<Catalog> {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Arc::new(Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("t_hero", "party"), unit("t_foe", "enemy")],
        "encounters": [{"id": "t_fight", "name": "Fight", "party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 8}],
        "rulesets": [{"id": "timeline", "name": "Timeline", "rules": ["round_intent", "plan_badges"]}]
    })))
}

fn ok(r: &Value) -> &Value {
    assert_eq!(r["ok"], json!(true), "{r}");
    r
}

#[test]
fn the_intents_reply_gains_both_sides_round_intents() {
    let mut host = Host::new(Battle::new(catalog()));
    ok(&host.handle(&json!({"cmd": "new", "encounter": "t_fight", "ruleset": "timeline", "plan_reveal": true})));
    assert!(host.commands.session.as_ref().unwrap().w.plan_reveal, "new takes plan_reveal");
    let r = host.handle(&json!({"cmd": "intents"}));
    let records = ok(&r)["round_intents"].as_array().expect("round_intents").clone();
    let sides: Vec<(String, String)> = records.iter().map(|x| (x["name"].as_str().unwrap().to_string(), x["side"].as_str().unwrap().to_string())).collect();
    assert_eq!(sides, [("t_hero".to_string(), "party".to_string()), ("t_foe".to_string(), "enemy".to_string())]);
    assert_eq!(records[1]["revealed"], json!(true), "plan_reveal reveals the enemy");
    let intel = host.handle(&json!({"cmd": "intel", "unit": "t_foe"}));
    assert_eq!(ok(&intel)["intel"], records[1], "intel is the unit's record");
    assert!(host.handle(&json!({"cmd": "intel"}))["error"].as_str().unwrap_or("").contains("needs a unit"));
}

#[test]
fn a_classic_reply_keeps_its_shape() {
    let mut host = Host::new(Battle::new(catalog()));
    ok(&host.handle(&json!({"cmd": "new", "encounter": "t_fight", "plan_reveal": true})));
    let r = host.handle(&json!({"cmd": "intents"}));
    assert!(ok(&r)["actions"].is_array());
    assert!(r.get("round_intents").is_none(), "classic: no round_intents");
    assert!(host.handle(&json!({"cmd": "intel", "unit": 0}))["error"].as_str().unwrap_or("").contains("plan_badges"));
}
