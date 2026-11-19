//! PR7-41 (PT-32) through the host on a scratch catalog (no shipped data): `retreat` ends a battle as fled while a
//! round is planned, in a fight whose round limit is a flight (`timeout_outcome` fled, the default); a fight whose
//! limit is a defeat refuses it, and so does a confirmed round; the retreat is recorded (a replay plays it again) and the
//! checkpoint result says fled.

use rpgkit_data::Catalog;
use rpgkit_host::{Battle, Host};
use serde_json::{json, Value};
use std::sync::Arc;

fn catalog() -> Arc<Catalog> {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 4000, "speed": 12, "attack_range": 1, "skill_cap": 5});
    let enc = |id: &str, timeout: Option<&str>| {
        let mut e = json!({"id": id, "name": id, "party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 3});
        if let Some(t) = timeout {
            e["timeout_outcome"] = json!(t);
        }
        e
    };
    Arc::new(Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("t_hero", "party"), unit("t_foe", "enemy")],
        "encounters": [enc("t_free", None), enc("t_stand", Some("lost"))]
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
fn a_retreat_while_planning_ends_the_fight_as_fled() {
    let mut h = Host::new(Battle::new(catalog()));
    ok(&h.handle(&json!({"cmd": "new", "encounter": "t_free"})));
    assert!(h.commands.session.as_ref().unwrap().can_retreat(), "round 1 planned, the limit is a flight");
    let r = h.handle(&json!({"cmd": "retreat"}));
    assert_eq!(ok(&r)["state"]["outcome"], json!("fled"), "{r}");
    let s = h.commands.session.as_ref().unwrap();
    assert!(s.combat_over && s.final_outcome() == "fled" && s.resolver.log.last().is_some_and(|l| l.starts_with("[Retreat]")));
    assert!(!s.can_retreat());
    refused(&h.handle(&json!({"cmd": "retreat"})), "Retreat only while planning");
    assert!(h.history.iter().any(|r| r["cmd"] == json!("retreat")), "recorded for a replay");
    assert!(h.checkpoints.last().is_some_and(|c| c.to_string().contains("fled")), "the checkpoint says fled: {:?}", h.checkpoints.last());
}

#[test]
fn a_fight_whose_limit_is_a_defeat_or_a_confirmed_round_refuses_it() {
    let mut h = Host::new(Battle::new(catalog()));
    ok(&h.handle(&json!({"cmd": "new", "encounter": "t_stand"})));
    refused(&h.handle(&json!({"cmd": "retreat"})), "allows no retreat");
    assert!(h.commands.session.as_ref().unwrap().outcome.is_empty(), "the fight goes on");
    let mut h = Host::new(Battle::new(catalog()));
    ok(&h.handle(&json!({"cmd": "new", "encounter": "t_free"})));
    ok(&h.handle(&json!({"cmd": "confirm"})));
    refused(&h.handle(&json!({"cmd": "retreat"})), "Retreat only while planning");
}

/// PR7-37 (PT-02, PT-24): a fight that runs to its round limit ends with no verdict of its own: `timed_out` (it reads
/// "Out of time", whatever its `timeout_outcome` settles it as); a retreat is a verdict, never a time out.
#[test]
fn the_round_limit_is_a_time_out_and_a_retreat_is_not() {
    for (enc, settles) in [("t_free", "fled"), ("t_stand", "lost")] {
        let mut h = Host::new(Battle::new(catalog()));
        ok(&h.handle(&json!({"cmd": "new", "encounter": enc})));
        assert!(!h.commands.session.as_ref().unwrap().timed_out(), "{enc}: under way");
        ok(&h.handle(&json!({"cmd": "autoplay"})));
        let s = h.commands.session.as_ref().unwrap();
        assert_eq!(s.final_outcome(), settles, "{enc}: the 4000 HP sides outlast the round limit");
        assert!(s.timed_out() && s.outcome.is_empty(), "{enc}: the limit ended it");
    }
    let mut h = Host::new(Battle::new(catalog()));
    ok(&h.handle(&json!({"cmd": "new", "encounter": "t_free"})));
    ok(&h.handle(&json!({"cmd": "retreat"})));
    let s = h.commands.session.as_ref().unwrap();
    assert!(s.final_outcome() == "fled" && !s.timed_out(), "a retreat is fled, not out of time");
}
