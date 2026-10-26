//! ENCOUNTER2-H through the host on a scratch catalog (no shipped data): `stay_pick {unit}` names the holder of a hold
//! escape once, before round 1 is confirmed, and is recorded for replays (H1); `confirm`, `step` and `resolve` refuse
//! until then; `autoplay` picks by `pick_default`; `state` gains `hold` only for a hold escape; the checkpoint result at
//! the round limit is the encounter's `timeout_outcome` (S2).

use rpgkit_data::Catalog;
use rpgkit_host::{outcome_of, Battle, Host};
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
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 4000, "speed": 12, "attack_range": 1, "skill_cap": 5});
    let base = json!({"party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 2, "objective": "escape", "exits": [[11, 4]]});
    let with = |extra: Value| {
        let mut e = base.clone();
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    Arc::new(Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy")],
        "encounters": [
            with(json!({"id": "t_hold", "name": "Hold", "timeout_outcome": "lost", "stay_behind": {"mode": "hold", "tiles": [[2, 1]], "tiles_label": "the winch"}})),
            with(json!({"id": "t_plain", "name": "Plain"}))
        ]
    })))
}

fn ok(r: &Value) -> &Value {
    assert_eq!(r["ok"], json!(true), "{r}");
    r
}

fn refused(r: &Value, text: &str) {
    assert_eq!(r["ok"], json!(false), "{r}");
    assert!(r["error"].as_str().unwrap_or("").contains(text), "{r}");
}

#[test]
fn the_holder_is_picked_once_before_round_one_is_confirmed() {
    let mut h = Host::new(Battle::new(catalog()));
    let started = h.handle(&json!({"cmd": "new", "encounter": "t_hold"}));
    let hold = ok(&started)["state"]["hold"].clone();
    assert_eq!((hold["label"].clone(), hold["holder"].clone(), hold["open"].clone(), hold["tiles"].clone()), (json!("the winch"), Value::Null, json!(false), json!([[2, 1]])));
    for cmd in ["confirm", "step", "resolve"] {
        refused(&h.handle(&json!({"cmd": cmd})), "pick who holds the winch first");
    }
    refused(&h.handle(&json!({"cmd": "stay_pick"})), "stay_pick needs a unit");
    refused(&h.handle(&json!({"cmd": "stay_pick", "unit": 2})), "Pick a hero on the field");
    let picked = h.handle(&json!({"cmd": "stay_pick", "unit": 1}));
    assert_eq!(ok(&picked)["holder"], json!(1));
    assert_eq!(picked["state"]["hold"]["holder"], json!(1));
    refused(&h.handle(&json!({"cmd": "stay_pick", "unit": 0})), "already holds the winch");
    ok(&h.handle(&json!({"cmd": "confirm"})));
    // the pick is in the history, so the requests replay to the same holder
    let history: Vec<Value> = h.handle(&json!({"cmd": "history"}))["history"].as_array().unwrap().clone();
    assert!(history.iter().any(|r| r["cmd"] == json!("stay_pick") && r["unit"] == json!(1)), "{history:?}");
    let mut again = Host::new(Battle::new(catalog()));
    for req in &history {
        again.handle(req);
    }
    assert_eq!(again.commands.session.as_ref().unwrap().w.hold.as_ref().unwrap().holder, Some(1));
}

#[test]
fn autoplay_picks_by_default_and_the_round_limit_gives_the_timeout_outcome() {
    let mut h = Host::new(Battle::new(catalog()));
    ok(&h.handle(&json!({"cmd": "new", "encounter": "t_hold"})));
    let played = h.handle(&json!({"cmd": "autoplay", "preset": "defensive"}));
    assert_eq!(ok(&played)["state"]["hold"]["holder"], json!(0), "highest_hp on equal HP: the first hero");
    // nobody holds the winch (the presets do not walk to it before ENCOUNTER2-HA), so the gate stays shut and no foe of
    // 4000 HP falls in two rounds: the rounds run out
    let s = h.commands.session.as_ref().unwrap();
    assert_eq!(s.outcome, "", "no verdict of the session's own");
    assert!(!s.can_advance_round(), "autoplay runs to the round limit");
    assert_eq!(outcome_of(s)["result"], json!("lost"), "timeout_outcome lost");
    let last = h.handle(&json!({"cmd": "history"}))["checkpoints"].as_array().unwrap().last().cloned().unwrap();
    assert_eq!(last["state"]["result"], json!("lost"), "the last checkpoint carries it: {last}");
}

#[test]
fn a_battle_without_a_hold_keeps_its_state_shape() {
    let mut h = Host::new(Battle::new(catalog()));
    let started = h.handle(&json!({"cmd": "new", "encounter": "t_plain"}));
    assert!(ok(&started)["state"].get("hold").is_none());
    refused(&h.handle(&json!({"cmd": "stay_pick", "unit": 0})), "no tile to hold");
    ok(&h.handle(&json!({"cmd": "confirm"})));
    assert_eq!(outcome_of(h.commands.session.as_ref().unwrap())["result"], json!(""), "undecided while it goes on");
}
