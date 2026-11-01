//! INPUT1-U host requests on a scratch catalog (no shipped data): `pick` appends a pick (the first one sets the intent,
//! kind and target), `unpick` and `clear` take picks back, `autofill` switches the ghost fill, `pick_options` lists a
//! target's menu rows without changing anything, and `round_plan` reports each member's picks, first ghost entry,
//! Auto-fill and AP forecast (INPUT1_TASKS 1.3), never an enemy. A refused pick changes nothing and is not recorded; the
//! history replays to the same plan.

use rpgkit_data::Catalog;
use rpgkit_host::{Battle, Host};
use serde_json::{json, Value};
use std::sync::Arc;

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
        json!({"id": "t_flank", "name": "Flank", "kind": "ATTACK", "round_intent": "maneuver", "level": 1, "blocks": 1, "ap_cost": 2, "power_mult": 0.6, "target_mode": "ENEMY", "position": "flank", "reach_max": 2}),
    ];
    let unit = |id: &str, side: &str| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 300, "speed": 12, "attack_range": 1, "skill_cap": 5, "deck": {"order": {"assault": ["t_cleave", "strike"], "guard": ["active_guard"]}}})
    };
    let ruleset = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "ap_cost"], "basics": ["move", "rest", "strike", "active_guard"]});
    let enc = json!({"id": "t_duel", "name": "Duel", "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 8, "ap_regen": 4, "max_rounds": 8});
    let units = [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy")];
    Arc::new(Catalog::from_value(&json!({"skills": skills, "units": units, "rulesets": [ruleset], "encounters": [enc]})))
}

fn host() -> Host {
    let mut h = Host::new(Battle::new(catalog()));
    let r = h.handle(&json!({"cmd": "new", "encounter": "t_duel", "ruleset": "timeline"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    h
}

fn row(h: &mut Host, unit: usize) -> Value {
    let rp = h.handle(&json!({"cmd": "round_plan"}))["round_plan"].clone();
    rp.as_array().unwrap().iter().find(|r| r["unit"] == json!(unit)).cloned().unwrap()
}

fn history(h: &mut Host) -> Vec<Value> {
    h.handle(&json!({"cmd": "history"}))["history"].as_array().unwrap().clone()
}

/// `pick` sets the intent and target from the skill, the plan holds the pick then the ghost, and `round_plan` gives the
/// picks, `ghost_from`, `autofill` and `ap` (now - spent = left, next); `manual` is true with a pick.
#[test]
fn a_pick_is_planned_and_reported() {
    let mut h = host();
    let r = h.handle(&json!({"cmd": "pick", "unit": 0, "target": 2, "pick": {"skill": "t_cleave"}}));
    assert_eq!((r["ok"].clone(), r["planned"].clone(), r["dropped"].clone()), (json!(true), json!(true), json!([])), "{r}");
    let hero = row(&mut h, 0);
    assert_eq!((hero["intent"].as_str(), hero["target"].clone(), hero["manual"].as_bool(), hero["autofill"].as_bool()), (Some("assault"), json!(2), Some(true), Some(true)));
    assert_eq!(hero["picks"], json!([{"skill": "t_cleave"}]));
    let entries = r["plan"]["t_hero"]["entries"].as_array().unwrap();
    let cleave = entries.iter().position(|e| e["skill"] == "t_cleave").expect("the pick is planned");
    assert_eq!(hero["ghost_from"], json!(cleave + 1), "the ghost starts after the pick (its approach is the pick's)");
    let ap = &hero["ap"];
    let spent: i64 = entries.iter().map(|e| e["ap"].as_i64().unwrap()).sum();
    assert_eq!((ap["now"].as_i64(), ap["spent"].as_i64(), ap["left"].as_i64()), (Some(8), Some(spent), Some(8 - spent)));
    assert_eq!(ap["picked"].as_i64().unwrap() + ap["filled"].as_i64().unwrap(), spent);
    assert_eq!(ap["gain"].as_i64().unwrap(), ap["next"].as_i64().unwrap() - ap["left"].as_i64().unwrap());
    assert_eq!(ap["max"], json!(8));
    let rp = h.handle(&json!({"cmd": "round_plan"}))["round_plan"].clone();
    assert!(rp.as_array().unwrap().iter().all(|r| r["side"] == "party"), "never an enemy");
    // a Maneuver pick on a fresh battle: the row's kind becomes the member's
    let mut h = host();
    let r = h.handle(&json!({"cmd": "pick", "unit": 1, "target": 2, "pick": {"skill": "t_flank", "kind": "engage"}}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(row(&mut h, 1)["maneuver"], json!("engage"));
}

/// A refused pick is an error, leaves the plan and the round plan as they were and is not recorded.
#[test]
fn a_refused_pick_changes_nothing() {
    let mut h = host();
    h.handle(&json!({"cmd": "pick", "unit": 0, "target": 2, "pick": {"skill": "t_cleave"}}));
    let (plan0, row0, hist0) = (h.handle(&json!({"cmd": "plan_view"}))["plan"].clone(), row(&mut h, 0), history(&mut h).len());
    let r = h.handle(&json!({"cmd": "pick", "unit": 0, "target": "self", "pick": {"skill": "active_guard"}}));
    assert_eq!((r["ok"].clone(), r["error"].as_str()), (json!(false), Some("Only Assault skills this round (Cleave set it)")), "{r}");
    assert!(h.handle(&json!({"cmd": "pick", "unit": 0, "pick": {"skill": "nope"}}))["error"].as_str().unwrap().contains("unknown skill"));
    assert_eq!((h.handle(&json!({"cmd": "plan_view"}))["plan"].clone(), row(&mut h, 0), history(&mut h).len()), (plan0, row0, hist0));
}

/// `unpick` takes the last pick back, `clear` every pick (the planner's own plan again); `autofill` off rests the free
/// blocks after the picks; `pick_options` reads the menu without changing anything; the history replays.
#[test]
fn unpick_clear_autofill_options_and_replay() {
    let mut h = host();
    h.handle(&json!({"cmd": "pick", "unit": 0, "target": 2, "pick": {"skill": "t_cleave"}}));
    h.handle(&json!({"cmd": "pick", "unit": 0, "target": 2, "pick": "rest"}));
    assert_eq!(row(&mut h, 0)["picks"], json!([{"skill": "t_cleave"}, "rest"]));
    let r = h.handle(&json!({"cmd": "unpick", "unit": 0}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(row(&mut h, 0)["picks"], json!([{"skill": "t_cleave"}]));

    let r = h.handle(&json!({"cmd": "autofill", "on": false}));
    assert_eq!((r["ok"].clone(), r["autofill"].clone()), (json!(true), json!(false)), "{r}");
    let plan = h.handle(&json!({"cmd": "plan_view", "unit": 0}))["plan"]["t_hero"]["entries"].clone();
    let from = row(&mut h, 0)["ghost_from"].as_u64().unwrap() as usize;
    let ghost: Vec<&str> = plan.as_array().unwrap()[from..].iter().map(|e| e["skill"].as_str().unwrap()).collect();
    assert!(!ghost.is_empty() && ghost.iter().all(|s| *s == "rest"), "Auto-fill off: the ghost rests: {ghost:?}");

    let before = (h.handle(&json!({"cmd": "plan_view"}))["plan"].clone(), row(&mut h, 0), history(&mut h).len());
    let opts = h.handle(&json!({"cmd": "pick_options", "unit": 0, "target": 2}));
    let rows = opts["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "{opts}");
    assert_eq!(rows.last().unwrap()["picks"], json!(["rest"]), "Rest last");
    for r in rows {
        for key in ["picks", "intent", "kind", "ap", "blocks", "problem", "drops"] {
            assert!(r.get(key).is_some(), "{key} in {r}");
        }
        assert!(r["problem"].is_null(), "after the first pick only rows that can be picked: {r}");
    }
    assert_eq!((h.handle(&json!({"cmd": "plan_view"}))["plan"].clone(), row(&mut h, 0), history(&mut h).len()), before, "read-only");

    let r = h.handle(&json!({"cmd": "clear", "unit": 0}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(row(&mut h, 0)["picks"], json!([]));
    h.handle(&json!({"cmd": "autofill", "on": true}));
    h.handle(&json!({"cmd": "pick", "unit": 1, "target": 2, "pick": {"skill": "strike"}}));

    let hist = history(&mut h);
    let cmds: Vec<&str> = hist.iter().map(|r| r["cmd"].as_str().unwrap()).collect();
    assert_eq!(cmds, ["new", "pick", "pick", "unpick", "autofill", "clear", "autofill", "pick"], "pick, unpick and autofill are recorded; pick_options is not");
    let mut again = Host::new(Battle::new(catalog()));
    for req in &hist {
        assert_eq!(again.handle(req)["ok"], json!(true), "{req}");
    }
    for u in [0, 1] {
        assert_eq!(row(&mut again, u), row(&mut h, u), "the replay reaches the same round plan");
    }
    assert_eq!(again.handle(&json!({"cmd": "plan_view"}))["plan"], h.handle(&json!({"cmd": "plan_view"}))["plan"]);
}
