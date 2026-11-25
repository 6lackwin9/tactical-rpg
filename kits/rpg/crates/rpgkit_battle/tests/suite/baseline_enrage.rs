//! ENRAGE2 (PR7-55, `docs/specs/ENRAGE2.md` EN1-EN5) on a scratch catalog built here (no shipped data): from the
//! ruleset's `enrage_from_round` every enemy on the board is Empowered at the round's start by its tier's step times the
//! rounds since (at most `enrage_max_stacks`); heroes never enrage; a larger phase enrage stays and the two never add; an
//! enraged enemy reads as enraged (intent revealed); the log names it once; a ruleset without the numbers has none.
//! EN9, EN10, EN12: a fight whose `timeout_outcome` is `escaped` ends so at its cap, and the party may retreat from it.

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::session::{EncounterDef, Phase, Session, ENRAGE_STATUS};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

const ASH: UnitId = 0;
const MOB: UnitId = 1;
const BOSS: UnitId = 2;
const FROM: i32 = 3;
const STEP_MINION: i32 = 10;
const STEP_BOSS: i32 = 20;
const MAX_STACKS: i32 = 2;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY", "ap_cost": 1}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str, tier: &str| json!({"id": id, "name": id, "side": side, "tier": tier, "stat_mode": "fixed", "hp": 9000, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("k_ash", "party", "hero"), unit("k_mob", "enemy", "minion"), unit("k_boss", "enemy", "boss")],
    }))
}

/// A fight of tough units (nobody falls in the rounds played) with `phases`, under the enrage numbers when `on`.
fn session(phases: Value, on: bool) -> Session {
    session_with(phases, on, json!({}))
}

fn session_with(phases: Value, on: bool, extra: Value) -> Session {
    let mut row = json!({"id": "enr", "name": "The Long Fight", "rows": vec![".".repeat(12); 9], "party": ["k_ash"], "enemies": ["k_mob", "k_boss"],
        "phases": phases, "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 12, "rng_seed": 7});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    let def = EncounterDef::from_obj(row.as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    if on {
        let n = &mut s.w.ruleset.numbers;
        n.enrage_from_round = FROM;
        n.enrage_step = [("minion".to_string(), STEP_MINION), ("boss".to_string(), STEP_BOSS)].into_iter().collect();
        n.enrage_max_stacks = MAX_STACKS;
    }
    for (c, cell) in [(ASH, V2::new(0, 0)), (MOB, V2::new(11, 8)), (BOSS, V2::new(11, 0))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.replan_enemies();
    s
}

/// Plays rounds until `round` has begun.
fn to_round(s: &mut Session, round: i32) {
    while s.current_round < round {
        assert!(s.confirm_plan(), "round {} confirms", s.current_round);
        while s.phase != Phase::Replay {
            s.step_next();
        }
        assert!(s.can_advance_round(), "round {} ends with both sides standing", s.current_round);
        s.advance_round();
    }
}

fn power(s: &Session, c: UnitId) -> i32 {
    let st = &s.w.units[c].st;
    if st.active.contains(ENRAGE_STATUS) {
        st.magnitudes.get(ENRAGE_STATUS).copied().unwrap_or(0)
    } else {
        0
    }
}

#[test]
fn enemies_enrage_by_tier_from_the_round_and_stop_at_the_steps() {
    let mut s = session(json!([]), true);
    assert_eq!(s.enrage_round(), Some(FROM));
    to_round(&mut s, FROM - 1);
    assert_eq!((power(&s, MOB), power(&s, BOSS)), (0, 0), "nobody before the round");
    assert!(!s.is_enraged(BOSS));
    for (round, stacks) in [(FROM, 1), (FROM + 1, 2), (FROM + 2, MAX_STACKS)] {
        to_round(&mut s, round);
        assert_eq!((power(&s, MOB), power(&s, BOSS)), (STEP_MINION * stacks, STEP_BOSS * stacks), "round {round}");
        assert_eq!(power(&s, ASH), 0, "a hero never enrages");
        assert!(s.is_enraged(MOB) && s.is_enraged(BOSS) && !s.is_enraged(ASH));
    }
    let cells: Vec<_> = s.log.iter().filter(|e| e.outcome == "ENRAGE").collect();
    assert_eq!(cells.len(), 1, "the log names it once");
    assert_eq!(cells[0].round, FROM);
}

#[test]
fn a_larger_phase_enrage_stays_and_the_two_never_add() {
    let phases = json!([{"id": "early", "name": "Early Rage", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "enrage", "unit": "k_boss", "power": 30}]}]);
    let mut s = session(phases, true);
    to_round(&mut s, FROM);
    assert_eq!(power(&s, BOSS), 30, "the larger phase enrage stays (20 < 30), not 50");
    to_round(&mut s, FROM + 1);
    assert_eq!(power(&s, BOSS), STEP_BOSS * 2, "the baseline takes over once larger");
}

#[test]
fn a_ruleset_without_the_numbers_has_no_baseline_enrage() {
    let mut s = session(json!([]), false);
    assert_eq!(s.enrage_round(), None);
    to_round(&mut s, FROM + 2);
    assert_eq!((power(&s, MOB), power(&s, BOSS)), (0, 0));
    assert!(s.log.iter().all(|e| e.outcome != "ENRAGE"));
}

#[test]
fn an_escape_fight_ends_escaped_at_its_cap_and_allows_a_retreat() {
    let cap = 2;
    let mut s = session_with(json!([]), true, json!({"max_rounds": cap, "timeout_outcome": "escaped"}));
    assert!(s.can_retreat(), "EN10: the party may flee at a planning step");
    for _ in 0..cap {
        assert!(s.confirm_plan());
        while s.phase != Phase::Replay {
            s.step_next();
        }
        if s.can_advance_round() {
            s.advance_round();
        }
    }
    assert_eq!(s.final_outcome(), rpgkit_battle::encounter::ESCAPED);
    assert!(s.timed_out());
    let mut s = session_with(json!([]), true, json!({"timeout_outcome": "escaped"}));
    s.retreat().expect("a retreat");
    assert_eq!(s.final_outcome(), rpgkit_battle::encounter::FLED, "a retreat is a flight");
}
