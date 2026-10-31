//! CAMPAIGN1-SIM: `sim deadlines` on scratch worlds built here, the expectations worked out by hand from the model
//! (CAMPAIGN1_TASKS 4.3) and the manifest's rest and defeat rules: day 1 at the hub, legs of 1 day each.

use rpgkit_sim::deadlines::{self, Plan};
use rpgkit_sim::SimGame;
use rpgkit_world::Data;
use serde_json::{json, Value};

/// The spec's hand example: hub `h`, a line h - n1 - n2 - n3 - n4 - n5 - n6 of 1-day legs (so `n2` is 2 legs out, `n3`
/// 3, `n4` 4), main quests M2 at n2, M3 at n3 (side quest S between them), M5 at n4; checkpoints on M2 (act 1, day
/// `d2`) and M5 (act 1, day `d5`). Rest and retreat 1 day each; the hub is the only rest place.
fn line_world(rest: Value, d2: i64, d5: i64) -> Value {
    let loc = |id: &str, rest: bool| json!({"id": id, "rest": rest});
    let route = |id: &str, a: &str, b: &str| json!({"id": id, "from": a, "to": b, "time": 1});
    json!({
        "manifest": {
            "id": "vale",
            "campaign": {"start_act": 1},
            "rest": rest,
            "defeat": {"mode": "retreat", "to": "h", "days": 1}
        },
        "own": {
            "locations": [loc("h", true), loc("n1", false), loc("n2", false), loc("n3", false), loc("n4", false), loc("n5", false), loc("n6", false)],
            "routes": [route("r1", "h", "n1"), route("r2", "n1", "n2"), route("r3", "n2", "n3"), route("r4", "n3", "n4"), route("r5", "n4", "n5"), route("r6", "n5", "n6")],
            "encounters": [{"id": "e5", "location": "n4"}],
            "quests": [
                {"id": "m2", "tags": ["main"], "steps": [{"location": "n2"}]},
                {"id": "s1", "steps": [{"location": "n6"}]},
                {"id": "m3", "track": "main", "steps": [{"location": "n3"}]},
                {"id": "m5", "tags": ["main"], "steps": [{"encounter": "e5"}]},
                {"id": "m9", "tags": ["main"], "steps": [{"location": "n6"}]}
            ],
            "checkpoints": [
                {"id": "cp_a", "act": 1, "quest": "m2", "deadline": d2},
                {"id": "cp_b", "act": 1, "quest": "m5", "deadline": d5},
                {"id": "cp_c", "act": 1, "met_when": [{"key": "vale.x"}], "deadline": 30}
            ]
        }
    })
}

/// The worlds as the loader gives them: every row stamped with its `_world` and `_qid`.
fn data(mut worlds: Value) -> Data {
    for (w, world) in worlds.as_object_mut().expect("worlds").iter_mut() {
        for rows in world["own"].as_object_mut().expect("own").values_mut() {
            for row in rows.as_array_mut().expect("rows") {
                let id = row["id"].as_str().unwrap_or_default().to_string();
                row["_world"] = json!(w);
                row["_qid"] = json!(format!("{w}:{id}"));
            }
        }
    }
    Data::from_values(&json!({}), &worlds, &json!({}))
}

fn rest_rule() -> Value {
    json!({"days": 1, "cost": 1, "where": "rest_places", "on_enter": false, "needs_upkeep": false})
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn plan(d: &Data, losses: i64) -> Plan {
    deadlines::plan(d, "vale", losses).expect("a plan")
}

fn days(p: &Plan, id: &str) -> (i64, i64) {
    p.checkpoints.iter().find(|c| c.id == id).expect("the checkpoint").days.clone().expect("days")
}

#[test]
fn worst_and_best_days_follow_the_model_by_hand() {
    let d = data(json!({"vale": line_world(rest_rule(), 14, 38)}));
    let p = plan(&d, 1);
    assert_eq!(p.hub, "vale:h");
    assert_eq!((p.rest_days, p.retreat_days), (1, 1));
    let ids: Vec<&str> = p.missions.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["m2", "m3", "m5"], "main quests (tag or track) in data order, up to the act's last checkpoint quest");
    let outs: Vec<Option<i64>> = p.missions.iter().map(|m| m.out).collect();
    assert_eq!(outs, [Some(2), Some(3), Some(4)], "legs out; m5's node is its encounter's location");
    // worst, by hand (time t, day t + 1): m2 out 2, retreat 1, rest 1, out 2 -> won at t 6 (day 7); back 2, rest 1 ->
    // t 9 (day 10). m3: out 3, retreat 1, rest 1, out 3 -> t 17 (day 18); back 3, rest 1 -> t 21 (day 22). m5 (last):
    // out 4, retreat 1, rest 1, out 4 -> t 31 (day 32). The spec's 10, 22, 32 are m2 and m3 home and m5 won.
    let worst: Vec<Option<i64>> = p.missions.iter().map(|m| m.worst).collect();
    assert_eq!(worst, [Some(7), Some(18), Some(32)]);
    assert_eq!(p.missions[0].worst_home, Some(10));
    assert_eq!(p.missions[1].worst_home, Some(22));
    // best: out and back, no rests: m2 won t 2 (day 3), back -> t 4; m3 won t 7 (day 8), back -> t 10; m5 won t 14
    let best: Vec<Option<i64>> = p.missions.iter().map(|m| m.best).collect();
    assert_eq!(best, [Some(3), Some(8), Some(15)]);
    assert_eq!(days(&p, "cp_a"), (3, 7));
    assert_eq!(days(&p, "cp_b"), (15, 32));
    assert_eq!(p.checkpoints[0].slack(), Some(14 - 7));
    assert_eq!(p.checkpoints[1].slack(), Some(38 - 32));
    assert!(p.checkpoints[2].days.as_ref().is_err_and(|e| e.contains("met_when")), "{:?}", p.checkpoints[2]);
}

#[test]
fn losses_scale_the_worst_case_only() {
    let d = data(json!({"vale": line_world(rest_rule(), 14, 38)}));
    let (zero, two) = (plan(&d, 0), plan(&d, 2));
    // no losses: the worst case is the best one plus a rest after each win (m2 home t 5, m3 won t 8 -> day 9)
    assert_eq!(days(&zero, "cp_a"), (3, 3));
    assert_eq!(zero.missions[1].worst, Some(9));
    // each extra loss costs retreat + rest + out: m2 +4 -> day 11
    assert_eq!(days(&two, "cp_a"), (3, 11));
    assert_eq!(days(&two, "cp_a").0, days(&zero, "cp_a").0, "the best case has no losses");
}

#[test]
fn a_rest_the_rule_refuses_at_the_hub_costs_no_days() {
    let none = json!({"days": 1, "where": "none"});
    let d = data(json!({"vale": line_world(none, 14, 38)}));
    let p = plan(&d, 1);
    assert_eq!(p.rest_days, 0);
    // m2: out 2, retreat 1, out 2 -> t 5 (day 6)
    assert_eq!(days(&p, "cp_a"), (3, 6));
}

#[test]
fn the_command_prints_the_table_and_flags_a_missed_deadline() {
    let d = data(json!({"vale": line_world(rest_rule(), 6, 38)}));
    let out = rpgkit_sim::run(&d, &argv("deadlines world=vale"), &SimGame::default());
    assert_eq!(out[0], "ARGS mode deadlines, world vale, losses 1");
    assert!(out[1].starts_with("DEADLINES vale: hub vale:h"), "{out:?}");
    let cp = out.iter().find(|l| l.starts_with("cp_a")).expect("the cp_a line");
    assert!(cp.ends_with("X MISSED in the worst case"), "{cp}");
    let again = rpgkit_sim::run(&d, &argv("deadlines world=vale"), &SimGame::default());
    assert_eq!(out, again, "deterministic");
}

#[test]
fn worlds_without_checkpoints_are_reported_cleanly() {
    let d = data(json!({"plain": {"manifest": {"id": "plain"}, "own": {}}}));
    let out = rpgkit_sim::run(&d, &argv("deadlines"), &SimGame::default());
    assert_eq!(out, ["ARGS mode deadlines, world all, losses 1", "DEADLINES: no world has campaign checkpoints (nothing to plan)"]);
    let one = rpgkit_sim::run(&d, &argv("deadlines world=plain"), &SimGame::default());
    assert_eq!(one[1], "DEADLINES plain: no campaign checkpoints");
}

#[test]
fn an_unreached_node_and_a_missing_retreat_are_reported() {
    let mut w = line_world(rest_rule(), 14, 38);
    w["own"]["routes"] = json!([{"id": "r1", "from": "h", "to": "n1", "time": 1}]);
    let d = data(json!({"vale": w.clone()}));
    let p = plan(&d, 1);
    let why = p.checkpoints[0].days.clone().expect_err("no path to n2");
    assert!(why.contains("m2") && why.contains("vale:n2"), "{why}");
    w["manifest"]["defeat"] = json!({});
    let d = data(json!({"vale": w}));
    let err = deadlines::plan(&d, "vale", 1).expect_err("no retreat hub");
    assert!(err.contains("retreat"), "{err}");
}

#[test]
fn deadlines_keys_and_positionals_are_checked() {
    let d = data(json!({"vale": line_world(rest_rule(), 14, 38)}));
    let game = SimGame::default();
    let err = |line: &str| rpgkit_sim::run(&d, &argv(line), &game)[0].clone();
    assert_eq!(err("deadlines losses=10"), "SIM argument error: losses must be 0..9 (got '10')");
    assert_eq!(err("deadlines 5"), "SIM argument error: deadlines takes no level or seeds");
    assert_eq!(err("duel losses=2"), "SIM argument error: losses= is a deadlines key (mode deadlines only)");
    assert_eq!(err("deadlines losses=3"), "ARGS mode deadlines, world all, losses 3");
}

/// CAMPAIGN1-TM: in a world with travel modes every leg is planned in the default mode, so each mission's legs take
/// the mode's days more (the line world's legs are 1 day: out = legs, in the mode legs x (1 + days)).
#[test]
fn legs_are_planned_in_the_default_travel_mode() {
    const MODE_DAYS: i64 = 1;
    let plain = plan(&data(json!({"vale": line_world(rest_rule(), 14, 38)})), 1);
    let mut world = line_world(rest_rule(), 14, 38);
    world["manifest"]["travel_modes"] = json!({"default": "stealth", "modes": [{"id": "forced", "name": "Forced"}, {"id": "stealth", "name": "Stealth", "days": MODE_DAYS}]});
    let moded = plan(&data(json!({"vale": world})), 1);
    assert_eq!(plain.mode, None);
    assert_eq!(moded.mode, Some(("Stealth".to_string(), MODE_DAYS)));
    for (a, b) in plain.missions.iter().zip(&moded.missions) {
        assert_eq!(b.out, a.out.map(|legs| legs * (1 + MODE_DAYS)), "{}", a.id);
    }
    let first = deadlines::plan_lines(&moded).remove(0);
    assert!(first.ends_with("legs in the travel mode Stealth (+1 day a leg)"), "{first}");
    assert!(!deadlines::plan_lines(&plain)[0].contains("travel mode"), "no modes: the line as before");
}
