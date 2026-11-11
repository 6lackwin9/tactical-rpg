//! PR5-25 (`FB-187` (5)) host requests on a scratch catalog (no shipped data): a party member a unit track's departure
//! stage turned `hostile_all` is reachable by an ally's SUPPORT `unmark` skill (the priest's Inoculated keystone) and
//! by an `unmark` item, as the planner allows them: `pick_options` on it lists the unmark rows (not a heal), `pick`
//! plans the unmark as a Support round on it, a heal pick is refused, and `items` with it as the target offers the
//! unmark item only.

use rpgkit_data::Catalog;
use rpgkit_host::{Battle, Host};
use serde_json::{json, Value};
use std::sync::Arc;

const MARK: &str = "t_mark";

fn catalog() -> Arc<Catalog> {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
        json!({"id": "t_inoculate", "name": "Inoculate", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ALLY", "reach_min": 0, "reach_max": 9, "unmark": {"mark": MARK}, "uses_per_battle": 1}),
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ALLY", "reach_min": 0, "reach_max": 9, "heal_base": 20}),
    ];
    let items = vec![
        json!({"id": "t_salt", "name": "Salt", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "use_blocks": 1, "use_reach": 9, "unmark": {"mark": MARK}}),
        json!({"id": "t_potion", "name": "Potion", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "use_blocks": 1, "use_reach": 9, "heal": 20}),
    ];
    let tracks = vec![json!({"id": "t_track", "name": "Track", "mark": MARK, "cause": "t_cause", "stages": [{"name": "One"}, {"name": "Two", "departure": "lost"}]})];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 300, "speed": 12, "attack_range": 1, "skill_cap": 5});
    let ruleset = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "ap_cost", "triggers"], "basics": ["move", "rest", "strike", "active_guard"]});
    let enc = json!({"id": "t_duel", "name": "Duel", "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 8, "ap_regen": 4, "max_rounds": 8});
    let units = [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy")];
    Arc::new(Catalog::from_value(&json!({"skills": skills, "items": items, "unit_tracks": tracks, "units": units, "rulesets": [ruleset], "encounters": [enc]})))
}

/// A duel whose hero (unit 0) a breach took to the track's departure stage: turned on everyone; the mate (unit 1)
/// carries a Salt and a Potion.
fn host() -> Host {
    let mut h = Host::new(Battle::new(catalog()));
    let r = h.handle(&json!({"cmd": "new", "encounter": "t_duel", "ruleset": "timeline"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let s = h.commands.session.as_mut().expect("a battle");
    s.w.units[0].tracks = Some([("t_track".to_string(), 1)].into_iter().collect());
    s.w.marks.record(0, MARK, 1);
    let parts = rpgkit_battle::mark::sync_tracks(&mut s.w, 0, MARK);
    assert!(parts.iter().any(|p| p == "t_hero turns on everyone"), "{parts:?}");
    for id in ["t_salt", "t_potion"] {
        let item = s.w.cat.items[id].clone();
        s.w.units[1].inventory.pouch.push(item);
    }
    s.w.add_item_skills();
    h
}

fn picks_of(rows: &Value) -> Vec<Value> {
    rows.as_array().unwrap().iter().filter(|r| r["problem"].is_null()).map(|r| r["picks"][0].clone()).collect()
}

#[test]
fn the_menu_and_pick_reach_the_turned_hero_with_an_unmark_only() {
    let mut h = host();
    let rows = h.handle(&json!({"cmd": "pick_options", "unit": 1, "target": 0}));
    assert_eq!(rows["ok"], json!(true), "{rows}");
    let open = picks_of(&rows["rows"]);
    assert!(open.contains(&json!({"skill": "t_inoculate"})) && open.contains(&json!({"item": "t_salt"})), "{rows}");
    assert!(!open.contains(&json!({"skill": "t_mend"})) && !open.contains(&json!({"item": "t_potion"})), "no heal on it: {rows}");
    let items = h.handle(&json!({"cmd": "items", "unit": 1, "target": 0}));
    let ids: Vec<&str> = items["items"].as_array().unwrap().iter().filter_map(|i| i["item"].as_str()).collect();
    assert_eq!(ids, ["t_salt"], "{items}");
    let r = h.handle(&json!({"cmd": "pick", "unit": 1, "target": 0, "pick": {"skill": "t_mend"}}));
    assert_eq!(r["ok"], json!(false), "a heal is refused: {r}");
    let r = h.handle(&json!({"cmd": "pick", "unit": 1, "target": 0, "pick": {"skill": "t_inoculate"}}));
    assert_eq!((r["ok"].clone(), r["planned"].clone()), (json!(true), json!(true)), "{r}");
    let entries = r["plan"]["t_mate"]["entries"].as_array().unwrap();
    assert!(entries.iter().any(|e| e["skill"] == "t_inoculate" && e["target"] == "t_hero"), "{r}");
    let rp = h.handle(&json!({"cmd": "round_plan"}))["round_plan"].clone();
    let mate = rp.as_array().unwrap().iter().find(|r| r["unit"] == json!(1)).cloned().unwrap();
    assert_eq!((mate["intent"].as_str(), mate["target"].clone()), (Some("support"), json!(0)), "{mate}");
}

/// PR5-26 (audit B10): the replies carry the PK1/PK2 state, each key only when set: `state` gives the turned hero its
/// marks, tracks and `outcome_of` keeps them; `skills` lists Inoculate's `unmark` and its use limit with the uses left
/// (one fewer once used this battle); a unit with none of it carries none of the keys.
#[test]
fn the_replies_carry_the_marks_tracks_and_use_limits() {
    let mut h = host();
    let st = h.handle(&json!({"cmd": "state"}))["state"].clone();
    let (hero, mate) = (st["units"][0].clone(), st["units"][1].clone());
    assert_eq!(hero["marks"], json!([[MARK, 1]]), "{hero}");
    assert!(hero["tracks"]["t_track"].as_i64().is_some_and(|n| n >= 2), "{hero}");
    for k in ["marks", "tracks", "skill_uses", "channel", "channeled", "forbid_hands", "pierce_melee"] {
        assert!(mate.get(k).is_none(), "the mate holds no {k}: {mate}");
    }
    let s = h.commands.session.as_ref().unwrap();
    let out = rpgkit_host::outcome_of(s);
    assert_eq!(out["units"][0]["marks"], hero["marks"]);
    assert!(out["units"][1].get("marks").is_none());
    let row = |h: &mut Host| h.handle(&json!({"cmd": "skills", "unit": 1}))["skills"].as_array().unwrap().iter().find(|r| r["skill"] == "t_inoculate").cloned().expect("Inoculate");
    let ino = row(&mut h);
    assert_eq!((ino["unmark"].clone(), ino["uses_per_battle"].clone(), ino["uses_left"].clone()), (json!(MARK), json!(1), json!(1)), "{ino}");
    h.commands.session.as_mut().unwrap().w.skill_uses.insert((1, "t_inoculate".to_string()), 1);
    assert_eq!(row(&mut h)["uses_left"], json!(0));
    let st = h.handle(&json!({"cmd": "state"}))["state"].clone();
    assert_eq!(st["units"][1]["skill_uses"], json!({"t_inoculate": 1}));
    let strike = h.handle(&json!({"cmd": "skills", "unit": 1}))["skills"].as_array().unwrap().iter().find(|r| r["skill"] == "strike").cloned().unwrap();
    assert!(strike.get("uses_per_battle").is_none() && strike.get("unmark").is_none() && strike.get("channeled").is_none(), "{strike}");
}
