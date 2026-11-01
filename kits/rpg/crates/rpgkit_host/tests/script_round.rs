//! ENCOUNTER2-C3 through the host on a scratch catalog (no shipped data): in a scripted, locked round (SR5) every
//! planning request is refused with "Round <n> is scripted." and records nothing, `confirm` answers ok, `step`,
//! `resolve`, `advance` and `autoplay` play on; the `state` reply carries `script` {round, lock, banner} only while a
//! round plays scripted; a recording (the history) replays the script from the data (SR7). Under classic the script is
//! ignored and nothing is refused (SR8).

use rpgkit_data::Catalog;
use rpgkit_host::{Battle, Host};
use serde_json::{json, Value};
use std::sync::Arc;

const REFUSED: [&str; 11] = ["preset", "plan", "add_step", "undo", "clear", "round_intent", "manual", "fill", "pick", "unpick", "autofill"];

fn catalog() -> Arc<Catalog> {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
        json!({"id": "t_cleave", "name": "Cleave", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 1.5, "target_mode": "ENEMY"}),
    ];
    let unit = |id: &str, side: &str| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 300, "speed": 12, "attack_range": 1, "skill_cap": 5, "deck": {"order": {"assault": ["t_cleave", "strike"], "guard": ["active_guard"]}}})
    };
    let ruleset = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "ap_cost"], "basics": ["move", "rest", "strike", "active_guard"]});
    let script = json!([{"round": 1, "lock": true, "others": "auto", "banner": "Watch them", "entries": [
        {"unit": "party:t_hero", "target": "self", "picks": [{"skill": "active_guard"}], "fill": "rest"},
        {"unit": "t_foe", "target": "self", "picks": [{"skill": "active_guard"}], "fill": "auto"}
    ]}]);
    let enc = json!({"id": "t_scripted", "name": "Scripted", "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 8, "ap_regen": 4, "max_rounds": 3, "script": script});
    let units = [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy")];
    Arc::new(Catalog::from_value(&json!({"skills": skills, "units": units, "rulesets": [ruleset], "encounters": [enc]})))
}

fn ok(r: &Value) -> &Value {
    assert_eq!(r["ok"], json!(true), "{r}");
    r
}

fn history(h: &mut Host) -> Vec<Value> {
    h.handle(&json!({"cmd": "history"}))["history"].as_array().unwrap().clone()
}

#[test]
fn a_locked_round_refuses_planning_and_plays_on() {
    let mut h = Host::new(Battle::new(catalog()));
    let started = h.handle(&json!({"cmd": "new", "encounter": "t_scripted", "ruleset": "timeline"}));
    assert_eq!(ok(&started)["state"]["script"], json!({"round": 1, "lock": true, "banner": "Watch them"}));
    assert_eq!(started["state"]["plan_confirmed"], json!(true), "confirmed at the round start");
    let plan = h.handle(&json!({"cmd": "plan_view"}))["plan"].clone();
    for cmd in REFUSED {
        let r = h.handle(&json!({"cmd": cmd, "unit": 0, "pick": "rest", "steps": [], "step": {"mode": "self", "skill": "rest"}, "on": false}));
        assert_eq!((r["ok"].clone(), r["error"].clone()), (json!(false), json!("Round 1 is scripted.")), "{cmd}");
    }
    assert_eq!(h.handle(&json!({"cmd": "plan_view"}))["plan"], plan, "nothing changed");
    let r = h.handle(&json!({"cmd": "confirm"}));
    ok(&r);
    let cmds: Vec<String> = history(&mut h).iter().map(|r| r["cmd"].as_str().unwrap().to_string()).collect();
    assert!(!cmds.iter().any(|c| REFUSED.contains(&c.as_str())), "a locked round records no planning request: {cmds:?}");
    ok(&h.handle(&json!({"cmd": "step"})));
    ok(&h.handle(&json!({"cmd": "resolve"})));
    let next = h.handle(&json!({"cmd": "advance"}));
    assert!(ok(&next)["state"].get("script").is_none(), "round 2 plays unscripted: the state keeps its shape");
    ok(&h.handle(&json!({"cmd": "pick", "unit": 0, "target": "self", "pick": {"skill": "active_guard"}})));
    ok(&h.handle(&json!({"cmd": "autoplay"})));

    // the history replays the battle: the script comes from the data again
    let hist = history(&mut h);
    let mut again = Host::new(Battle::new(catalog()));
    for req in &hist {
        ok(&again.handle(req));
    }
    assert_eq!(again.handle(&json!({"cmd": "log"}))["log"], h.handle(&json!({"cmd": "log"}))["log"]);
    assert_eq!(again.handle(&json!({"cmd": "state"}))["state"], h.handle(&json!({"cmd": "state"}))["state"]);
}

#[test]
fn autoplay_plays_a_locked_round_as_scripted() {
    let mut a = Host::new(Battle::new(catalog()));
    ok(&a.handle(&json!({"cmd": "new", "encounter": "t_scripted", "ruleset": "timeline"})));
    let scripted = a.handle(&json!({"cmd": "intents"}))["actions"].clone();
    ok(&a.handle(&json!({"cmd": "autoplay", "rounds": 1})));
    let mut b = Host::new(Battle::new(catalog()));
    ok(&b.handle(&json!({"cmd": "new", "encounter": "t_scripted", "ruleset": "timeline"})));
    ok(&b.handle(&json!({"cmd": "resolve"})));
    assert_eq!(a.handle(&json!({"cmd": "log"}))["log"], b.handle(&json!({"cmd": "log"}))["log"], "autoplay leaves the script's plan alone");
    let hero: Vec<&Value> = scripted.as_array().unwrap().iter().filter(|x| x["unit"] == json!(0)).collect();
    assert!(!hero.is_empty(), "{scripted}");
}

#[test]
fn classic_ignores_the_script() {
    let mut h = Host::new(Battle::new(catalog()));
    let started = h.handle(&json!({"cmd": "new", "encounter": "t_scripted"}));
    assert!(ok(&started)["state"].get("script").is_none());
    assert_eq!(started["state"]["plan_confirmed"], json!(false));
    ok(&h.handle(&json!({"cmd": "preset", "preset": "balanced"})));
    ok(&h.handle(&json!({"cmd": "autoplay"})));
}
