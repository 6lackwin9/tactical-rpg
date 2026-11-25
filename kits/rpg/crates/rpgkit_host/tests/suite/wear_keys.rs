//! PR7-3 (review N3, `docs/sessions/PK2_GRAFTS.md` 5 and 7) through the host on a scratch catalog: a unit's state row
//! (`state`, the inspect read) carries `wear` (slot save key -> {durability, worn, dulled}, the keys of the world
//! settle's `wear` rows) for each weapon it tracks and `attacked` (the weapon families it attacked with, the settle's
//! mastery use); the round checkpoint (`outcome_of`, the result read) keeps `wear`. A unit that tracks nothing, and a
//! battle without them, keep their shape. The values come from the battle's own `Wear` (durability left, spent, at 0
//! dulled), set here on the session.

use rpgkit_data::Catalog;
use rpgkit_host::{outcome_of, Battle, Host};
use rpgkit_rules::gear::{Slot, Wear};
use serde_json::{json, Value};
use std::sync::Arc;

fn catalog() -> Arc<Catalog> {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Arc::new(Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("t_hero", "party"), unit("t_foe", "enemy")],
        "encounters": [{"id": "t_fight", "name": "Fight", "party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 8}],
        "rulesets": [{"id": "worn", "name": "Worn", "rules": ["durability"]}]
    })))
}

fn ok(r: &Value) -> &Value {
    assert_eq!(r["ok"], json!(true), "{r}");
    r
}

fn hero_row(host: &mut Host) -> Value {
    ok(&host.handle(&json!({"cmd": "state"})))["state"]["units"][0].clone()
}

#[test]
fn a_tracked_weapon_reads_its_wear_and_dulled_state() {
    let mut host = Host::new(Battle::new(catalog()));
    ok(&host.handle(&json!({"cmd": "new", "encounter": "t_fight", "ruleset": "worn"})));
    let row = hero_row(&mut host);
    assert!(row.get("wear").is_none() && row.get("attacked").is_none(), "nothing tracked: no keys {row}");
    let start = 3;
    {
        let s = host.commands.session.as_mut().unwrap();
        s.w.units[0].wear.insert(Slot::MainHand, Wear { start, spent: 1 });
        s.w.weapon_attacks.insert(0, vec!["t_blade".into()]);
    }
    let row = hero_row(&mut host);
    assert_eq!(row["wear"], json!({"main_hand": {"durability": start - 1, "worn": 1, "dulled": false}}), "{row}");
    assert_eq!(row["attacked"], json!(["t_blade"]));
    // spent past the start: no durability left, dulled (the settle clamps the instance at 0)
    host.commands.session.as_mut().unwrap().w.units[0].wear.insert(Slot::MainHand, Wear { start, spent: start + 1 });
    let row = hero_row(&mut host);
    assert_eq!(row["wear"]["main_hand"], json!({"durability": 0, "worn": start + 1, "dulled": true}));
    let foe = ok(&host.handle(&json!({"cmd": "state"})))["state"]["units"][1].clone();
    assert!(foe.get("wear").is_none() && foe.get("attacked").is_none(), "the foe tracks nothing: {foe}");
    // the result read keeps the wear (the mastery use stays an inspect key)
    let out = outcome_of(host.commands.session.as_ref().unwrap());
    assert_eq!(out["units"][0]["wear"], row["wear"]);
    assert!(out["units"][0].get("attacked").is_none() && out["units"][1].get("wear").is_none());
}

#[test]
fn a_battle_without_wear_keeps_its_shape() {
    let mut host = Host::new(Battle::new(catalog()));
    ok(&host.handle(&json!({"cmd": "new", "encounter": "t_fight"})));
    let state = ok(&host.handle(&json!({"cmd": "state"})))["state"].clone();
    for u in state["units"].as_array().unwrap() {
        assert!(u.get("wear").is_none() && u.get("attacked").is_none(), "{u}");
    }
    let out = outcome_of(host.commands.session.as_ref().unwrap());
    assert!(out["units"].as_array().unwrap().iter().all(|u| u.get("wear").is_none()));
}
