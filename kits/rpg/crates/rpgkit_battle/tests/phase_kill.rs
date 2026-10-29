//! ENCOUNTER2-K on a scratch catalog built here (no shipped data): the phase `kill` effect (docs/specs/ENCOUNTER2.md
//! P1), phase groups (P2) and hp_below on a phase-wave unit (P5). Expectations come from the rules: at fire time every
//! active, placed, unreserved unit of the refs drops to 0 HP through the world's damage path (journaled), never a hero;
//! `within` > 0 keeps only units at most that many tiles from the `by` unit on the field; the fates read Killed (so
//! REWARD1's key loot, which pays on a win when every unit of its ref is Killed, pays unchanged); one EVENT cell per
//! effect that took HP, actor `by` else the encounter; the kill credits no unit (the metrics balance with its HP as
//! the struck units' `scripted` loss); the user-confirmed ruling (encounter::KILL_OVERRIDES_UNKILLABLE) takes an
//! unkillable unit down. When a phase fires, every unfired phase of its group drops.

use indexmap::IndexMap;
use rpgkit_battle::encounter::KILL_OVERRIDES_UNKILLABLE;
use rpgkit_battle::grid::V2;
use rpgkit_battle::log::{kill_line, Entry, PHASE_KILL};
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::session::{EncounterDef, Fate, Phase, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

/// The roster order: party, then enemies (brute, two guards, the boss), then phase waves.
const ASH: UnitId = 0;
const BRYN: UnitId = 1;
const BRUTE: UnitId = 2;
const GUARD: UnitId = 3;
const GUARD_B: UnitId = 4;
const BOSS: UnitId = 5;

/// Heroes on the left edge; the boss with one guard beside it and one far away, the brute in a corner.
const CELLS: [V2; 6] = [V2 { x: 0, y: 4 }, V2 { x: 0, y: 5 }, V2 { x: 8, y: 0 }, V2 { x: 9, y: 4 }, V2 { x: 11, y: 8 }, V2 { x: 10, y: 4 }];

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, name: &str, side: &str| json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("k_ash", "Ash", "party"), unit("k_bryn", "Bryn", "party"), unit("k_brute", "Brute", "enemy"), unit("k_guard", "Guard", "enemy"), unit("k_boss", "Boss", "enemy"), unit("k_scout", "Scout", "enemy")],
    }))
}

/// A rout on an open floor with `phases` and `extra` merged over the row.
fn row(phases: Value, extra: Value) -> Value {
    let mut row = json!({"id": "cull", "name": "The Cull", "rows": vec![".".repeat(12); 9], "party": ["k_ash", "k_bryn"],
        "enemies": ["k_brute", "k_guard", "k_guard", "k_boss"], "phases": phases,
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 1337});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

/// The battle of `row` with the party and the enemies on CELLS.
fn session(row: Value) -> Session {
    let def = EncounterDef::from_obj(row.as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    place(&mut s);
    s.replan_enemies();
    s
}

fn place(s: &mut Session) {
    for (c, cell) in CELLS.into_iter().enumerate() {
        s.w.grid.positions[c] = Some(cell);
    }
}

/// Resolves the round with an empty party plan, puts the party and the enemies back on CELLS and starts the next
/// round (round-trigger phases fire at its start).
fn next_round(s: &mut Session) {
    assert!(s.confirm_plan(), "the plan is confirmed");
    while s.phase != Phase::Replay {
        s.step_next();
    }
    assert!(s.can_advance_round(), "the fight goes on");
    place(s);
    s.advance_round();
}

/// A round-2 phase `id` named Cull with `effects`.
fn at_round_two(id: &str, effects: Value) -> Value {
    json!({"id": id, "name": "Cull", "trigger": {"kind": "round", "round": 2}, "effects": effects})
}

fn kill_cells(s: &Session) -> Vec<&Entry> {
    s.log.iter().filter(|e| e.category == "EVENT" && e.subtype == PHASE_KILL).collect()
}

fn names(s: &Session, units: &[UnitId]) -> Vec<String> {
    units.iter().map(|&c| s.w.units[c].name.clone()).collect()
}

/// Takes three quarters of `c`'s HP (below every 50% trigger).
fn wound(s: &mut Session, c: UnitId) {
    let hp = s.w.units[c].hp;
    s.w.damage(c, hp - hp / 4, true);
}

#[test]
fn a_kill_with_two_refs_downs_every_placed_unit_of_them_once() {
    // a later phase holds a third guard back (reserved, off the board): the kill passes it by
    let phases = json!([
        at_round_two("cull", json!([{"type": "line", "text": "The cull begins."}, {"type": "kill", "units": ["k_brute", "k_guard"], "by": "k_boss"}])),
        {"id": "late", "name": "Late", "trigger": {"kind": "round", "round": 5}, "effects": [{"type": "wave", "units": ["k_guard"]}]},
    ]);
    let mut s = session(row(phases.clone(), json!({})));
    let late = s.phases.units_of("late");
    assert_eq!(late.len(), 1, "one reserved guard");
    next_round(&mut s);
    assert_eq!(s.phases.fired.get("cull"), Some(&(2, 0)), "the phase fired at round 2's start");
    for c in [BRUTE, GUARD, GUARD_B] {
        assert!(!s.w.units[c].is_alive(), "{} is down", s.w.units[c].name);
        assert!(s.w.grid.positions[c].is_some(), "the dead keep their tiles");
    }
    assert!(s.w.units[BOSS].is_alive() && s.w.units[ASH].is_alive() && s.w.units[BRYN].is_alive());
    assert!(s.w.units[late[0]].is_alive() && s.is_reserved(late[0]), "the reserved guard is not on the field yet");
    // one cell, after the phase's own, actor the by unit, the HP taken as its damage
    let cells = kill_cells(&s);
    assert_eq!(cells.len(), 1, "one event cell");
    let fallen = names(&s, &[BRUTE, GUARD, GUARD_B]);
    let refs: Vec<&str> = fallen.iter().map(String::as_str).collect();
    assert_eq!(cells[0].detail, kill_line(&refs));
    assert!(cells[0].detail.ends_with(" fall."));
    assert_eq!((cells[0].actor.as_str(), cells[0].actor_is_ally, cells[0].skill.as_str(), cells[0].value), ("Boss", false, "Cull", 3));
    assert!(cells[0].damage > 0);
    let at = s.log.iter().position(|e| e.subtype == PHASE_KILL).unwrap();
    assert_eq!((s.log[at - 1].subtype.as_str(), s.log[at - 1].detail.as_str()), ("Phase", "The cull begins."));
    assert!(s.events.contains(&cells[0].detail));
    // the fates: Killed for every fallen unit (REWARD1's key loot reads these), the reserved guard standing
    let fates = s.fates();
    let of = |r: &str| fates.iter().filter(|(x, _)| x == r).map(|(_, f)| *f).collect::<Vec<_>>();
    assert_eq!(of("k_brute"), vec![Fate::Killed]);
    assert_eq!(of("k_guard"), vec![Fate::Killed, Fate::Killed, Fate::Standing]);
    assert_eq!(of("k_boss"), vec![Fate::Standing]);
    // the same battle with the HP journal followed: each fall is journaled at 0 HP
    let mut t = session(row(phases, json!({})));
    t.w.hp_journal = Some(Vec::new());
    next_round(&mut t);
    let journal = t.w.hp_journal.clone().unwrap();
    for c in [BRUTE, GUARD, GUARD_B] {
        let last = journal.iter().rev().find(|(u, _)| *u == c).map(|(_, hp)| *hp);
        assert_eq!(last, Some(0), "{}'s fall is journaled", t.w.units[c].name);
    }
}

#[test]
fn within_keeps_only_units_near_the_by_unit() {
    let phases = json!([at_round_two("cull", json!([{"type": "kill", "units": ["k_guard"], "by": "k_boss", "within": 2}]))]);
    let mut s = session(row(phases, json!({})));
    next_round(&mut s);
    assert!(CELLS[GUARD].dist(CELLS[BOSS]) <= 2 && CELLS[GUARD_B].dist(CELLS[BOSS]) > 2, "the layout");
    assert!(!s.w.units[GUARD].is_alive(), "the guard beside the boss falls");
    assert!(s.w.units[GUARD_B].is_alive(), "the far guard stands");
    let cells = kill_cells(&s);
    assert_eq!(cells.len(), 1);
    let guard = s.w.units[GUARD].name.clone();
    assert_eq!((cells[0].detail.clone(), cells[0].target.clone()), (kill_line(&[guard.as_str()]), guard));
    // a by unit that is not on the field: nobody is within reach, nobody falls, no cell
    let phases = json!([at_round_two("cull", json!([{"type": "kill", "units": ["k_guard"], "by": "k_nobody", "within": 11}]))]);
    let mut s = session(row(phases, json!({})));
    next_round(&mut s);
    assert!(s.w.units[GUARD].is_alive() && s.w.units[GUARD_B].is_alive());
    assert!(kill_cells(&s).is_empty());
    assert!(s.phases.fired.contains_key("cull"), "the phase itself fired");
}

#[test]
fn a_kill_without_by_is_the_encounters_and_never_takes_a_hero() {
    let phases = json!([at_round_two("cull", json!([{"type": "kill", "units": ["k_ash", "k_brute"]}]))]);
    let mut s = session(row(phases, json!({})));
    next_round(&mut s);
    assert!(s.w.units[ASH].is_alive(), "a hero never falls to a kill");
    assert!(!s.w.units[BRUTE].is_alive());
    let cells = kill_cells(&s);
    assert_eq!((cells.len(), cells[0].actor.as_str(), cells[0].value), (1, "The Cull", 1));
}

#[test]
fn a_kill_takes_an_unkillable_unit_down_by_the_ruling() {
    assert!(KILL_OVERRIDES_UNKILLABLE, "the ruling (user-confirmed 2026-10-29)");
    let phases = json!([at_round_two("cull", json!([{"type": "kill", "units": ["k_boss"]}]))]);
    let mut s = session(row(phases, json!({"unkillable": ["k_boss"]})));
    assert_eq!(s.w.unkillable, vec![BOSS]);
    next_round(&mut s);
    assert!(!s.w.units[BOSS].is_alive(), "the scripted kill overrides unkillable");
    assert!(s.w.spared.is_empty(), "no SPARED record");
    assert_eq!(kill_cells(&s).len(), 1);
    assert!(!s.log.iter().any(|e| e.outcome == "SPARED"));
    // the other reading of the ruling, World::kill without the override: 1 HP and the S1 record
    let mut t = session(row(json!([]), json!({"unkillable": ["k_boss"]})));
    t.w.kill(BOSS, false);
    assert_eq!(t.w.units[BOSS].hp, 1);
    assert_eq!(t.w.spared, vec![BOSS]);
}

/// The metrics after a phase kill, the battle closed (`finish`): the HP balances with the kill's HP as the struck
/// units' `scripted` loss, no unit has a kill or an unexplained change, for either setting of the unkillable switch
/// (off: the unkillable boss keeps 1 HP, its SPARED cell right after the kill's).
#[test]
fn the_metrics_balance_after_a_phase_kill_with_either_switch() {
    for past in [true, false] {
        let phases = json!([at_round_two("cull", json!([{"type": "kill", "units": ["k_brute", "k_guard", "k_boss"]}]))]);
        let mut s = session(row(phases, json!({"unkillable": ["k_boss"]})));
        assert_eq!(s.kill_past_unkillable, KILL_OVERRIDES_UNKILLABLE);
        s.kill_past_unkillable = past;
        s.attach_metrics(BattleMetrics::new(), true);
        next_round(&mut s);
        let cells = kill_cells(&s);
        assert_eq!(cells.len(), 1, "past {past}: one kill cell");
        let taken = cells[0].damage;
        assert_eq!(s.w.units[BOSS].is_alive(), !past, "past {past}");
        let fallen = if past { names(&s, &[BRUTE, GUARD, GUARD_B, BOSS]) } else { names(&s, &[BRUTE, GUARD, GUARD_B]) };
        let refs: Vec<&str> = fallen.iter().map(String::as_str).collect();
        assert_eq!((cells[0].detail.clone(), cells[0].value), (kill_line(&refs), fallen.len() as i32), "past {past}");
        let at = s.log.iter().position(|e| e.subtype == PHASE_KILL).unwrap();
        if past {
            assert!(s.w.spared.is_empty() && !s.log.iter().any(|e| e.outcome == "SPARED"));
        } else {
            assert_eq!(s.w.units[BOSS].hp, 1);
            assert_eq!((s.log[at + 1].outcome.as_str(), s.log[at + 1].actor.as_str()), ("SPARED", "Boss"), "the SPARED cell follows");
        }
        assert!(s.confirm_plan());
        while s.phase != Phase::Replay {
            s.step_next();
        }
        if past {
            assert_eq!(s.outcome, "won", "every enemy down is a rout");
        }
        let mut m = s.metrics.take().unwrap();
        m.finish(&s);
        assert!(m.fail_lines.is_empty(), "past {past}: {:?}", m.fail_lines);
        assert!(m.units.values().all(|r| r.kills == 0), "past {past}: a scripted kill is nobody's");
        assert!(m.units.values().all(|r| r.other_loss == 0 && r.other_gain == 0), "past {past}: nothing unexplained");
        assert_eq!(m.scripted.values().sum::<i32>(), taken, "past {past}: the kill's HP is the struck units' scripted loss");
    }
}

/// The break of node 20 as two phases in one group: `first` on the boss (with or_round 4), `second` on the brute.
fn grouped() -> Value {
    json!([
        {"id": "first", "name": "Break", "group": "break", "trigger": {"kind": "hp_below", "unit": "k_boss", "pct": 50, "or_round": 4}, "effects": [{"type": "line", "text": "First."}]},
        {"id": "second", "name": "Break", "group": "break", "trigger": {"kind": "hp_below", "unit": "k_brute", "pct": 50}, "effects": [{"type": "line", "text": "Second."}]},
    ])
}

#[test]
fn the_phase_whose_trigger_is_met_first_fires_and_its_group_drops() {
    let mut s = session(row(grouped(), json!({})));
    wound(&mut s, BRUTE);
    next_round(&mut s);
    assert!(s.phases.fired.contains_key("second"), "the second's trigger was met first");
    assert!(s.phases.dropped.contains(&"first".to_string()), "its group-mate drops");
    // the first's own trigger later, and its or_round: it never fires
    wound(&mut s, BOSS);
    for _ in 0..3 {
        next_round(&mut s);
    }
    assert!(s.current_round >= 4);
    assert!(!s.phases.fired.contains_key("first"));
    assert_eq!(s.log.iter().filter(|e| e.subtype == "Phase").count(), 1, "one phase cell");
    // both triggers met at once: the first in list order fires, the other drops
    let mut s = session(row(grouped(), json!({})));
    wound(&mut s, BRUTE);
    wound(&mut s, BOSS);
    next_round(&mut s);
    assert!(s.phases.fired.contains_key("first") && !s.phases.fired.contains_key("second"));
    assert!(s.phases.dropped.contains(&"second".to_string()));
    // without a group both fire
    let mut ungrouped = grouped();
    for p in ungrouped.as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("group");
    }
    let mut s = session(row(ungrouped, json!({})));
    wound(&mut s, BRUTE);
    wound(&mut s, BOSS);
    next_round(&mut s);
    assert!(s.phases.fired.contains_key("first") && s.phases.fired.contains_key("second"));
}

#[test]
fn an_hp_below_trigger_may_watch_a_phase_wave_unit() {
    // P5: the scout arrives with a round-2 wave; once it is below half the watch fires
    let phases = json!([
        {"id": "wave", "name": "Wave", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": ["k_scout"]}]},
        {"id": "watch", "name": "Watch", "trigger": {"kind": "hp_below", "unit": "k_scout", "pct": 50}, "effects": [{"type": "line", "text": "Watch."}]},
    ]);
    let mut s = session(row(phases, json!({})));
    next_round(&mut s);
    assert!(s.phases.fired.contains_key("wave"));
    assert!(!s.phases.fired.contains_key("watch"), "the scout is whole");
    let scout = s.phases.units_of("wave")[0];
    assert!(!s.is_reserved(scout));
    wound(&mut s, scout);
    next_round(&mut s);
    assert!(s.phases.fired.contains_key("watch"));
}
