//! GEAR-REWORK1 (PR5-51a, `docs/sessions/U3_GEAR_REWORK.md` 1) through the host on a scratch catalog (no shipped data):
//! `Switch Weapon` is retired, so the battle command set has no weapon switch (a `switch_weapon` request is an unknown
//! command, refused with nothing changed) and the plan a fresh battle offers lists no such action.

use rpgkit_data::Catalog;
use rpgkit_host::{Battle, Host};
use serde_json::json;
use std::sync::Arc;

fn catalog() -> Arc<Catalog> {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Arc::new(Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("t_hero", "party"), unit("t_foe", "enemy")],
        "encounters": [{"id": "t_plain", "name": "Plain", "party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 2}]
    })))
}

#[test]
fn there_is_no_switch_weapon_command() {
    let mut h = Host::new(Battle::new(catalog()));
    let started = h.handle(&json!({"cmd": "new", "encounter": "t_plain"}));
    assert_eq!(started["ok"], json!(true), "{started}");
    let before = h.handle(&json!({"cmd": "state"}));
    let r = h.handle(&json!({"cmd": "switch_weapon", "unit": 0}));
    assert_eq!(r["ok"], json!(false), "{r}");
    assert!(r["error"].as_str().unwrap_or("").contains("unknown cmd 'switch_weapon'"), "{r}");
    assert_eq!(h.handle(&json!({"cmd": "state"})), before, "nothing changed");
    let text = started.to_string().to_lowercase();
    assert!(!text.contains("switch weapon") && !text.contains("switch_weapon"), "no switch action offered");
}
