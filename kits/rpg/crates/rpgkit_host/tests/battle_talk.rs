//! ENCOUNTER2-C1 through the host on a scratch catalog (no shipped data): the `state` reply carries `talks` (the cue
//! list {phase, dialogue, speaker, round, block}) only when the encounter has a phase `talk` effect (DT3), so every other
//! state keeps its shape; `step`, `resolve` and `autoplay` never wait for a talk.

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
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 4000, "speed": 12, "attack_range": 1, "skill_cap": 5});
    let base = json!({"party": ["t_hero"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 3});
    let with = |extra: Value| {
        let mut e = base.clone();
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    let phases = json!([
        {"id": "opening", "name": "Opening", "trigger": {"kind": "round", "round": 1}, "effects": [{"type": "talk", "dialogue": "dlg_open", "speaker": "t_hero"}]},
        {"id": "mid", "name": "Mid", "trigger": {"kind": "round", "round": 2, "block": 3}, "effects": [{"type": "line", "text": "Again."}, {"type": "talk", "dialogue": "dlg_mid", "speaker": ""}]}
    ]);
    Arc::new(Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("t_hero", "party"), unit("t_foe", "enemy")],
        "encounters": [
            with(json!({"id": "t_talk", "name": "Talk", "phases": phases})),
            with(json!({"id": "t_line", "name": "Line", "phases": [{"id": "mid", "name": "Mid", "trigger": {"kind": "round", "round": 2, "block": 3}, "effects": [{"type": "line", "text": "Again."}]}]}))
        ]
    })))
}

fn ok(r: &Value) -> &Value {
    assert_eq!(r["ok"], json!(true), "{r}");
    r
}

#[test]
fn the_state_lists_the_talks_only_when_the_encounter_has_one() {
    let mut h = Host::new(Battle::new(catalog()));
    let started = h.handle(&json!({"cmd": "new", "encounter": "t_talk"}));
    assert_eq!(ok(&started)["state"]["talks"], json!([{"phase": "opening", "dialogue": "dlg_open", "speaker": "t_hero", "round": 1, "block": 0}]));
    // autoplay plays on through the round-2 talk without waiting
    let played = h.handle(&json!({"cmd": "autoplay", "rounds": 2}));
    assert_eq!(ok(&played)["rounds"], json!(2));
    let talks = played["state"]["talks"].as_array().cloned().unwrap_or_default();
    assert_eq!(talks.len(), 2, "{talks:?}");
    assert_eq!(talks[1], json!({"phase": "mid", "dialogue": "dlg_mid", "speaker": "", "round": 2, "block": 3}));
    assert_eq!(ok(&h.handle(&json!({"cmd": "state"})))["state"]["talks"], played["state"]["talks"]);

    let mut plain = Host::new(Battle::new(catalog()));
    let started = plain.handle(&json!({"cmd": "new", "encounter": "t_line"}));
    assert!(ok(&started)["state"].get("talks").is_none(), "no talk effect: the state keeps its shape");
    let played = plain.handle(&json!({"cmd": "autoplay", "rounds": 2}));
    assert!(ok(&played)["state"].get("talks").is_none());
}
