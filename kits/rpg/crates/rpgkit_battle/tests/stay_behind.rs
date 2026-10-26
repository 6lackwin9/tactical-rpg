//! ENCOUNTER2-H on a scratch catalog built here (no shipped data): the hold-a-tile escape (`stay_behind` mode hold,
//! docs/specs/ENCOUNTER2.md 3.1), `unkillable` (S1), the timeout verdict in the session (S2) and the party fates (S3).
//! Expectations come from the rules: round 1 is not confirmed before someone holds; the pick is allowed once, while
//! round 1 is planned (H1); the gate is open while the holder is active on a hold tile, or with the holder down and
//! `fallback` any while any hero is (H2); a step onto an exit through a shut gate fails and the unit stays before it,
//! while planning still takes the Withdraw and the holder's own Withdraw is refused (H3); the escape is won at a block
//! end once the gate is open, `objective_count` heroes are out and every other one is out, down or holding, and the
//! holders stayed (H4); the last round over gives `timeout_outcome` (fled unless it names lost; H5, S2); damage leaves
//! an unkillable unit at 1 HP and the log one cell per unit (S1).

use indexmap::IndexMap;
use rpgkit_battle::encounter::PartyFate;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::PlanStep;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

/// The heroes in party order, then the foe (unit ids follow the roster: party, then enemies).
const ASH: UnitId = 0;
const BRYN: UnitId = 1;
const COLE: UnitId = 2;
const DARA: UnitId = 3;
const FOE: UnitId = 4;

/// The winch tiles, the exits beside the heroes and the foe's corner.
const WINCH: V2 = V2 { x: 10, y: 1 };
const SECOND: V2 = V2 { x: 10, y: 7 };
const LABEL: &str = "the winch";

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
        "units": [unit("s_ash", "Ash", "party"), unit("s_bryn", "Bryn", "party"), unit("s_cole", "Cole", "party"), unit("s_dara", "Dara", "party"), unit("s_brute", "Brute", "enemy")],
    }))
}

/// An open floor board (every tile walkable), so the tiles below are the rule's only constraint.
fn floor() -> Value {
    json!(vec![".".repeat(12); 9])
}

/// The escape `hold` with `stay_behind` merged over a hold of the winch (count 3 out of four heroes, exits beside the
/// heroes) and `extra` merged over the row.
fn row(stay: Value, extra: Value) -> Value {
    let mut sb = json!({"mode": "hold", "tiles": [[WINCH.x, WINCH.y]], "tiles_label": LABEL});
    for (k, v) in stay.as_object().unwrap() {
        sb[k] = v.clone();
    }
    let mut row = json!({"id": "hold", "name": "Hold", "rows": floor(), "party": ["s_ash", "s_bryn", "s_cole", "s_dara"], "enemies": ["s_brute"],
        "exits": [[11, 3], [11, 4], [11, 5]], "objective": "escape", "objective_count": 3, "stay_behind": sb,
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 4, "rng_seed": 1337});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

/// The battle of `row` with the units on `cells` (heroes in order, then the foe) and the foe's draft redone there.
fn session(row: Value, cells: [V2; 5]) -> Session {
    let def = EncounterDef::from_obj(row.as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in cells.into_iter().enumerate() {
        s.w.grid.positions[c] = Some(cell);
    }
    s.replan_enemies();
    s
}

/// Ash on the winch, Bryn, Cole and Dara beside the three exits, the foe far away in the opposite corner.
fn by_the_exits() -> [V2; 5] {
    [WINCH, V2::new(10, 3), V2::new(10, 4), V2::new(10, 5), V2::new(0, 8)]
}

/// Plans a Withdraw for each of `units` (each accepted by the planner).
fn withdraw(s: &mut Session, units: &[UnitId]) {
    for &c in units {
        let p = s.plan.add_step(&s.w, c, PlanStep::withdraw(), true);
        assert_eq!(p, "", "{} plans Withdraw", s.w.units[c].name);
    }
}

/// Confirms the plan and resolves the round (or the battle's end).
fn resolve(s: &mut Session) {
    assert!(s.confirm_plan(), "the plan is confirmed");
    while s.phase != Phase::Replay {
        s.step_next();
    }
}

fn fate_of(s: &Session, c: UnitId) -> PartyFate {
    s.party_fates().into_iter().find(|(u, _)| *u == c).map(|(_, f)| f).expect("a hero")
}

#[test]
fn round_one_waits_for_the_pick_and_the_pick_is_made_once() {
    let mut s = session(row(json!({}), json!({})), by_the_exits());
    assert!(s.needs_stay_pick(), "a hold escape waits for its holder");
    assert!(s.plan_is_valid(), "the plan itself is fine");
    assert!(!s.confirm_plan(), "round 1 is not confirmed before the pick");
    s.step_next();
    assert!(s.phase == Phase::Planning && !s.plan_confirmed, "stepping does not confirm it either");
    assert!(s.stay_pick(FOE).is_err(), "an enemy cannot hold");
    assert_eq!(s.stay_pick(ASH), Ok(()));
    assert_eq!(s.w.hold.as_ref().unwrap().holder, Some(ASH));
    assert!(!s.needs_stay_pick());
    assert_eq!(s.stay_pick(BRYN), Err(format!("Ash already holds {LABEL}.")), "a second pick is refused");
    assert!(s.confirm_plan(), "after the pick the plan is confirmed");
    // the same battle without a hold has nothing to pick
    let mut plain = session(row(json!({}), json!({"stay_behind": {}})), by_the_exits());
    assert!(!plain.needs_stay_pick() && plain.w.hold.is_none());
    assert!(plain.stay_pick(ASH).is_err());
    assert!(plain.confirm_plan());
}

#[test]
fn the_pick_is_refused_once_round_one_is_confirmed() {
    // the default pick (sims, autoplay), then confirm: no pick after that
    let mut s = session(row(json!({}), json!({})), by_the_exits());
    assert_eq!(s.stay_pick_default(), Some(ASH), "highest_hp on equal HP: the first hero");
    assert!(s.confirm_plan());
    assert!(s.stay_pick(BRYN).is_err());
    assert_eq!(s.stay_pick_default(), None, "no pick is due");
}

#[test]
fn the_default_pick_follows_pick_default() {
    let mut s = session(row(json!({}), json!({})), by_the_exits());
    s.w.units[COLE].hp += 10;
    assert_eq!(s.stay_pick_default(), Some(COLE), "highest_hp: the hero with the most HP now");
    let mut s = session(row(json!({"pick_default": "first"}), json!({})), by_the_exits());
    s.w.units[COLE].hp += 10;
    assert_eq!(s.stay_pick_default(), Some(ASH), "first: the first hero on the field");
}

#[test]
fn a_shut_gate_stops_a_hero_before_the_exit() {
    // Ash holds but stands off the winch: the gate is shut
    let mut cells = by_the_exits();
    cells[ASH] = V2::new(5, 4);
    let mut s = session(row(json!({}), json!({})), cells);
    s.stay_pick(ASH).unwrap();
    let gate = s.w.hold.clone().unwrap();
    assert!(!gate.is_open(&s.w));
    assert_eq!(s.gate_note(), format!("The gate is shut until Ash holds {LABEL}."), "the plan line warns");
    // the world step fails and the unit stays where it was
    let mut w = s.w.clone();
    assert!(!w.move_unit(COLE, V2::new(1, 0)), "a step onto the exit fails like a blocked tile");
    assert_eq!(w.grid.pos(COLE), Some(V2::new(10, 4)));
    assert!(!w.units[COLE].is_withdrawn);
    // planning still takes the Withdraw; resolving it leaves the heroes before the exits
    withdraw(&mut s, &[BRYN, COLE, DARA]);
    resolve(&mut s);
    for (c, cell) in [(BRYN, V2::new(10, 3)), (COLE, V2::new(10, 4)), (DARA, V2::new(10, 5))] {
        assert!(!s.w.units[c].is_withdrawn, "{} stayed in", s.w.units[c].name);
        assert_eq!(s.w.grid.pos(c), Some(cell), "{} stopped before the exit", s.w.units[c].name);
    }
    assert_eq!(s.outcome, "");
}

#[test]
fn the_holder_cannot_plan_withdraw() {
    let mut s = session(row(json!({}), json!({})), by_the_exits());
    s.stay_pick(ASH).unwrap();
    assert_eq!(s.plan.add_step(&s.w, ASH, PlanStep::withdraw(), true), format!("Ash holds {LABEL}."));
    assert_eq!(s.plan.add_step(&s.w, BRYN, PlanStep::withdraw(), true), "", "the others may");
    // nor take an exit itself: a hold tile beside an exit, the gate open, Ash may not step out while Cole may
    let mut s = session(row(json!({"tiles": [[10, 4]]}), json!({})), [V2::new(10, 4), V2::new(5, 3), V2::new(10, 3), V2::new(5, 5), V2::new(0, 8)]);
    s.stay_pick(ASH).unwrap();
    assert!(s.w.hold.as_ref().unwrap().is_open(&s.w));
    let mut w = s.w.clone();
    assert!(!w.move_unit(ASH, V2::new(1, 0)), "the holder never steps out");
    assert!(w.move_unit(COLE, V2::new(1, 0)) && w.units[COLE].is_withdrawn, "the gate is open for the others");
}

#[test]
fn the_holder_on_the_winch_lets_three_out_and_stays() {
    let mut s = session(row(json!({}), json!({})), by_the_exits());
    s.stay_pick(ASH).unwrap();
    assert!(s.w.hold.as_ref().unwrap().is_open(&s.w), "the holder is on the winch");
    assert_eq!(s.gate_note(), "");
    withdraw(&mut s, &[BRYN, COLE, DARA]);
    resolve(&mut s);
    assert_eq!(s.outcome, "won");
    assert_eq!(s.current_round, 1, "won in the round they got out");
    assert_eq!(s.objective.stayed, vec![ASH]);
    assert_eq!(s.party_fates(), vec![(ASH, PartyFate::Stayed), (BRYN, PartyFate::Escaped), (COLE, PartyFate::Escaped), (DARA, PartyFate::Escaped)]);
    assert_eq!(s.final_outcome(), "won");
}

#[test]
fn too_few_out_is_no_win() {
    // two out, the third keeps standing: count 3 is not met, so the battle goes on
    let mut s = session(row(json!({}), json!({})), by_the_exits());
    s.stay_pick(ASH).unwrap();
    withdraw(&mut s, &[BRYN, COLE]);
    resolve(&mut s);
    assert_eq!(s.outcome, "");
    assert!(s.objective.stayed.is_empty());
    assert_eq!(fate_of(&s, DARA), PartyFate::Standing);
    assert_eq!(fate_of(&s, ASH), PartyFate::Standing, "nobody stayed before the win");
}

/// Ash (the holder) down on the winch, Dara on the second hold tile, Bryn and Cole beside the exits; count 2.
fn holder_down(fallback: &str) -> Session {
    let stay = json!({"tiles": [[WINCH.x, WINCH.y], [SECOND.x, SECOND.y]], "fallback": fallback});
    let mut s = session(row(stay, json!({"objective_count": 2})), [WINCH, V2::new(10, 3), V2::new(10, 4), SECOND, V2::new(0, 8)]);
    s.stay_pick(ASH).unwrap();
    let hp = s.w.units[ASH].hp;
    s.w.damage(ASH, hp + 100, true);
    assert!(!s.w.units[ASH].is_alive(), "fixture: the holder is down");
    s
}

#[test]
fn with_fallback_any_a_second_hero_opens_the_gate() {
    let mut s = holder_down("any");
    assert!(s.w.hold.as_ref().unwrap().is_open(&s.w), "Dara holds a tile");
    withdraw(&mut s, &[BRYN, COLE]);
    resolve(&mut s);
    assert_eq!(s.outcome, "won");
    assert_eq!(s.objective.stayed, vec![DARA]);
    assert_eq!(s.party_fates(), vec![(ASH, PartyFate::Downed), (BRYN, PartyFate::Escaped), (COLE, PartyFate::Escaped), (DARA, PartyFate::Stayed)]);
}

#[test]
fn with_fallback_none_the_gate_stays_shut() {
    let mut s = holder_down("none");
    assert!(!s.w.hold.as_ref().unwrap().is_open(&s.w));
    let mut w = s.w.clone();
    for c in [BRYN, COLE] {
        assert!(!w.move_unit(c, V2::new(1, 0)), "no exit step succeeds");
    }
    withdraw(&mut s, &[BRYN, COLE]);
    resolve(&mut s);
    assert!(!s.w.units[BRYN].is_withdrawn && !s.w.units[COLE].is_withdrawn);
    assert_eq!(s.outcome, "");
}

#[test]
fn no_active_foe_still_wins_first_with_nobody_staying() {
    let mut s = session(row(json!({}), json!({})), by_the_exits());
    s.stay_pick(ASH).unwrap();
    let hp = s.w.units[FOE].hp;
    s.w.damage(FOE, hp + 100, true);
    resolve(&mut s);
    assert_eq!(s.outcome, "won");
    assert!(s.objective.stayed.is_empty());
    assert!(s.party_fates().iter().all(|(_, f)| *f == PartyFate::Standing));
}

/// Plays rounds with an empty party plan (and the default pick) until the last one is over.
fn to_the_end(s: &mut Session) {
    loop {
        s.stay_pick_default();
        resolve(s);
        assert_eq!(s.final_outcome() != "", !s.can_advance_round(), "a final outcome only once no round follows");
        if !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
}

#[test]
fn the_last_round_over_gives_the_timeout_outcome() {
    // Ash off the winch the whole time: nobody gets out, the rounds run out
    let mut cells = by_the_exits();
    cells[ASH] = V2::new(5, 4);
    for (extra, want) in [(json!({"timeout_outcome": "lost", "max_rounds": 2}), "lost"), (json!({"max_rounds": 2}), "fled"), (json!({"timeout_outcome": "fled", "max_rounds": 2}), "fled")] {
        let mut s = session(row(json!({}), extra.clone()), cells);
        assert_eq!(s.final_outcome(), "", "{extra}: undecided while it goes on");
        to_the_end(&mut s);
        assert_eq!(s.outcome, "", "{extra}: the session's own verdict is none");
        assert!(!s.combat_over);
        assert_eq!(s.final_outcome(), want, "{extra}");
    }
    // a battle without a stay-behind block reads its timeout_outcome too
    let mut s = session(row(json!({}), json!({"stay_behind": {}, "objective": "rout", "timeout_outcome": "lost", "max_rounds": 1})), cells);
    to_the_end(&mut s);
    assert_eq!(s.final_outcome(), "lost");
}

#[test]
fn an_unkillable_unit_keeps_one_hp_and_says_so_once() {
    let mut s = session(row(json!({}), json!({"objective": "rout", "stay_behind": {}, "unkillable": ["s_brute"]})), by_the_exits());
    assert_eq!(s.w.unkillable, vec![FOE]);
    for _ in 0..2 {
        let hp = s.w.units[FOE].hp;
        s.w.damage(FOE, hp + 100, true);
        assert_eq!(s.w.units[FOE].hp, 1, "lethal damage leaves 1 HP");
        assert!(s.w.units[FOE].is_alive());
    }
    assert_eq!(s.w.spared, vec![FOE], "recorded once");
    resolve(&mut s);
    let line = "Brute will not fall here.";
    let cells: Vec<_> = s.log.iter().filter(|e| e.category == "EVENT" && e.detail == line).collect();
    assert_eq!(cells.len(), 1, "one event cell");
    assert_eq!((cells[0].actor.as_str(), cells[0].round), ("Brute", 1));
    assert_eq!(s.events.iter().filter(|l| l.as_str() == line).count(), 1);
    assert_ne!(s.outcome, "won", "rout cannot end the fight while it stands");
    // heroes are never unkillable: the field names enemy-side units only
    let s = session(row(json!({}), json!({"unkillable": ["s_ash"]})), by_the_exits());
    assert!(s.w.unkillable.is_empty());
}

#[test]
fn a_battle_without_the_fields_plays_as_before() {
    // the plain escape: three out and the fourth out too is a win, as today (no gate, no holder, nobody stayed)
    let mut s = session(row(json!({}), json!({"stay_behind": {}})), [V2::new(11, 2), V2::new(10, 3), V2::new(10, 4), V2::new(10, 5), V2::new(0, 8)]);
    assert!(s.w.hold.is_none() && s.w.unkillable.is_empty() && s.stay.is_none());
    assert_eq!(s.timeout_outcome, "fled");
    withdraw(&mut s, &[ASH, BRYN, COLE, DARA]);
    resolve(&mut s);
    assert_eq!(s.outcome, "won");
    assert!(s.party_fates().iter().all(|(_, f)| *f == PartyFate::Escaped));
    // a stay-behind block on another objective is ignored (the checks refuse that row)
    let s = session(row(json!({}), json!({"objective": "rout"})), by_the_exits());
    assert!(s.stay.is_none() && s.w.hold.is_none());
}
