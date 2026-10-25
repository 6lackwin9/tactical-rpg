//! RULESET1-PL host requests on a scratch catalog (no shipped data): `round_intent` sets a member's intent, target,
//! kind and opener and rebuilds its auto plan (RULESET1_TASKS 1.8), `manual` and `fill` (MM1, MM3), `round_intents`
//! reports every unit's choice with the reason tags (PL6); the requests replay from the history; classic refuses them.

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
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 300, "speed": 12, "attack_range": 1, "skill_cap": 5, "deck": {"order": {"guard": ["active_guard"]}}});
    let ruleset = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "ap_cost"], "basics": ["move", "rest", "strike", "active_guard"]});
    let enc = json!({"id": "t_duel", "name": "Duel", "party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 3, "ap_regen": 4, "max_rounds": 8});
    Arc::new(Catalog::from_value(&json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_foe", "enemy")], "rulesets": [ruleset], "encounters": [enc]})))
}

fn host(ruleset: &str) -> Host {
    let mut h = Host::new(Battle::new(catalog()));
    let r = h.handle(&json!({"cmd": "new", "encounter": "t_duel", "ruleset": ruleset}));
    assert_eq!(r["ok"], json!(true), "{r}");
    h
}

fn hero_row(h: &mut Host) -> Value {
    h.handle(&json!({"cmd": "round_intents"}))["round_intents"][0].clone()
}

/// PL1, PL6, GD2: Guard on self with 3 AP plans Active Guard three times (opener, then follow-ups) and reports the
/// choice and the tags; an opener of another intent is not offered (planned false with the reason).
#[test]
fn round_intent_plans_and_reports() {
    let mut h = host("timeline");
    let r = h.handle(&json!({"cmd": "round_intent", "unit": 0, "intent": "guard", "target": "self"}));
    assert_eq!((r["ok"].clone(), r["planned"].clone()), (json!(true), json!(true)), "{r}");
    let entries = r["plan"]["t_hero"]["entries"].as_array().unwrap();
    assert_eq!(entries.iter().map(|e| e["skill"].as_str().unwrap()).collect::<Vec<_>>(), ["active_guard"; 3]);
    let row = hero_row(&mut h);
    assert_eq!((row["intent"].as_str(), row["target"].as_str(), row["manual"].as_bool()), (Some("guard"), Some("self"), Some(false)));
    assert_eq!(row["reasons"], json!(["opener", "follow-up 1", "follow-up 1"]));
    let r = h.handle(&json!({"cmd": "round_intent", "unit": 0, "opener": "strike"}));
    assert_eq!((r["planned"].clone(), r["reason"].as_str()), (json!(false), Some("Only Guard skills this round.")), "{r}");
    assert_eq!(hero_row(&mut h)["opener"], json!("strike"));
    let r = h.handle(&json!({"cmd": "round_intent", "unit": 0, "opener": ""}));
    assert_eq!(r["planned"], json!(true), "{r}");
    assert!(h.handle(&json!({"cmd": "round_intent", "unit": 0, "maneuver": "engage"}))["error"].as_str().unwrap_or("").contains("Maneuver kinds"));
    assert!(h.handle(&json!({"cmd": "round_intent", "unit": 1, "intent": "guard"}))["error"].as_str().unwrap_or("").contains("party unit"));
}

/// MM1, MM3: Manual keeps the steps (cleared here, a Rest added by hand), `fill` places the follow-ups after it, Manual
/// off rebuilds the auto plan; the history replays to the same choices and tags.
#[test]
fn manual_and_fill_replay() {
    let mut h = host("timeline");
    h.handle(&json!({"cmd": "round_intent", "unit": 0, "intent": "guard", "target": "self"}));
    let r = h.handle(&json!({"cmd": "manual", "unit": 0}));
    assert_eq!(r["round_intents"][0]["manual"], json!(true), "{r}");
    h.handle(&json!({"cmd": "clear", "unit": 0}));
    assert_eq!(h.handle(&json!({"cmd": "add_step", "unit": 0, "step": {"mode": "self", "skill": "rest"}}))["ok"], json!(true));
    let r = h.handle(&json!({"cmd": "fill", "unit": 0}));
    let names: Vec<&str> = r["plan"]["t_hero"]["entries"].as_array().unwrap().iter().map(|e| e["skill"].as_str().unwrap()).collect();
    assert_eq!(names, ["rest", "active_guard", "active_guard", "rest"], "{r}");
    let tags = hero_row(&mut h)["reasons"].clone();
    // the last free block: Active Guard needs 2 blocks (checked before its AP, FU2 b then c)
    assert_eq!(tags, json!(["", "follow-up 1", "follow-up 1", "rest: blocks 1 < 2"]));
    let history: Vec<Value> = h.handle(&json!({"cmd": "history"}))["history"].as_array().unwrap().clone();
    let mut again = Host::new(Battle::new(catalog()));
    for req in &history {
        again.handle(req);
    }
    assert_eq!(hero_row(&mut again), hero_row(&mut h), "the requests replay");
    let r = h.handle(&json!({"cmd": "manual", "unit": 0, "on": false}));
    assert_eq!((r["planned"].clone(), r["round_intents"][0]["reasons"].clone()), (json!(true), json!(["opener", "follow-up 1", "follow-up 1"])), "{r}");
}

/// A `round_intent` request is atomic: one that names a new intent and then a refused field leaves the battle exactly
/// as it was (intent, plan, state, events), is not recorded, and the history replays to the same battle.
#[test]
fn a_refused_round_intent_changes_nothing() {
    let mut h = host("timeline");
    h.handle(&json!({"cmd": "round_intent", "unit": 0, "intent": "guard", "target": "self"}));
    let snapshot = |h: &mut Host| {
        let events = h.commands.session.as_ref().unwrap().events.clone();
        (hero_row(h), h.handle(&json!({"cmd": "plan_view"}))["plan"].clone(), h.handle(&json!({"cmd": "state"}))["state"].clone(), events)
    };
    let before = snapshot(&mut h);
    for bad in [
        json!({"cmd": "round_intent", "unit": 0, "intent": "maneuver", "maneuver": "bogus"}),
        json!({"cmd": "round_intent", "unit": 0, "intent": "assault", "target": "self"}),
        json!({"cmd": "round_intent", "unit": 0, "intent": "support", "opener": "no_such_skill"}),
    ] {
        let r = h.handle(&bad);
        assert_eq!(r["ok"], json!(false), "{bad}: {r}");
        assert_eq!(snapshot(&mut h), before, "{bad} left the battle as it was");
    }
    let history: Vec<Value> = h.handle(&json!({"cmd": "history"}))["history"].as_array().unwrap().clone();
    assert!(history.iter().all(|q| q.get("maneuver").is_none() && q.get("opener").is_none()), "refused requests are not recorded");
    let mut again = Host::new(Battle::new(catalog()));
    for req in &history {
        again.handle(req);
    }
    assert_eq!(snapshot(&mut again), before, "the history replays to the same battle");
}

/// Classic has no round intents: the requests are refused and `round_intents` is empty.
#[test]
fn classic_refuses_round_intents() {
    let mut h = host("classic");
    assert!(h.handle(&json!({"cmd": "round_intent", "unit": 0, "intent": "guard"}))["error"].as_str().unwrap_or("").contains("no round intents"));
    assert_eq!(h.handle(&json!({"cmd": "manual", "unit": 0}))["ok"], json!(false));
    assert_eq!(h.handle(&json!({"cmd": "round_intents"}))["round_intents"], json!([]));
}
