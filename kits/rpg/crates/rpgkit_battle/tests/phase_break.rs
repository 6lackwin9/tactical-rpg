//! PHASEBREAK1 (PR7-56, `docs/specs/BOSSUI1.md` BU15, BU17) on a scratch catalog built here (no shipped data): a hit
//! that takes a unit to or below the threshold of its `hp_below` phase stops there (the rest is lost) and ends the
//! round at once: no later action of either side resolves or spends AP, the log's EVENT cell names the break and the
//! skipped blocks, and the phase fires at the next round's start. A `round` phase is unchanged; once the last
//! `hp_below` phase fired nothing stops a hit; a phase `vignette` only adds its cue.

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::PlanStep;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

const ASH: UnitId = 0;
const BRYN: UnitId = 1;
const BOSS: UnitId = 2;
const MAX: i32 = 5000;
/// The boss's phase at half its HP, so its threshold is `MAX / 2`.
const PCT: i32 = 50;
const FLOOR: i32 = MAX * PCT / 100;
const BLOCKS: i32 = 6;
const AP: i32 = 10;

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
    let unit = |id: &str, name: &str, side: &str, hp: i32| json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": hp, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("k_ash", "Ash", "party", 5000), unit("k_bryn", "Bryn", "party", 5000), unit("k_boss", "Boss", "enemy", MAX)],
    }))
}

/// A fight against the boss with `phases`.
fn session(phases: Value) -> Session {
    let row = json!({"id": "brk", "name": "The Break", "rows": vec![".".repeat(12); 9], "party": ["k_ash", "k_bryn"], "enemies": ["k_boss"],
        "phases": phases, "total_blocks": BLOCKS, "unit_ap": AP, "ap_regen": 0, "max_rounds": 6, "rng_seed": 1337});
    let def = EncounterDef::from_obj(row.as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(ASH, V2::new(10, 5)), (BRYN, V2::new(10, 3)), (BOSS, V2::new(10, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.replan_enemies();
    s
}

fn half_phase(extra: Value) -> Value {
    let mut fx = vec![json!({"type": "line", "text": "The boss changes."})];
    fx.extend(extra.as_array().cloned().unwrap_or_default());
    json!([{"id": "half", "name": "Second Form", "trigger": {"kind": "hp_below", "unit": "k_boss", "pct": PCT}, "effects": fx}])
}

/// Both heroes strike the boss in every block.
fn strike_all(s: &mut Session) {
    let strike = s.w.book().find("strike").expect("strike");
    for c in [ASH, BRYN] {
        for _ in 0..BLOCKS {
            let why = s.plan.add_step(&s.w, c, PlanStep::engage(strike, BOSS), true);
            assert!(why.is_empty(), "{why}");
        }
    }
}

fn resolve(s: &mut Session) {
    assert!(s.confirm_plan(), "the plan is confirmed");
    while s.phase != Phase::Replay {
        s.step_next();
    }
}

#[test]
fn a_hit_past_the_threshold_stops_there_and_ends_the_round() {
    let mut s = session(half_phase(json!([])));
    s.w.units[BOSS].hp = FLOOR + 1;
    strike_all(&mut s);
    resolve(&mut s);
    assert_eq!(s.w.units[BOSS].hp, FLOOR, "the hit stops at the threshold; the rest is lost");
    let breaks = s.phase_breaks().to_vec();
    assert_eq!(breaks.len(), 1, "{breaks:?}");
    let (unit, round, block) = breaks[0];
    assert_eq!((unit, round, block), (BOSS, 1, 1), "the first hit broke it in block 1");
    let cell = s.log.iter().find(|e| e.outcome == rpgkit_battle::session::PHASE_BREAK_OUTCOME).expect("the break's cell");
    assert_eq!((cell.actor.as_str(), cell.status.as_str()), ("Boss", "Blocks 2-6 skipped"));
    assert!(s.log.iter().all(|e| e.block <= block || e.round != round || e.category == "EVENT" && e.outcome != rpgkit_battle::session::PHASE_BREAK_OUTCOME), "no cell of a later block");
    // AP: each hero paid for its block-1 strike only (the later strikes never started), as the same round without the
    // phase stands after block 1
    let mut free = session(json!([]));
    free.w.units[BOSS].hp = FLOOR + 1;
    strike_all(&mut free);
    assert!(free.confirm_plan());
    while free.phase != Phase::Replay && free.current_block < 2 {
        free.step_next();
    }
    for c in [ASH, BRYN] {
        assert_eq!(s.w.units[c].ap, free.w.units[c].ap, "{}: the AP after block 1", s.w.units[c].name);
    }
    while free.phase != Phase::Replay {
        free.step_next();
    }
    assert!(free.w.units[ASH].ap < s.w.units[ASH].ap, "without the break the later strikes spend AP");
    assert!(!s.phases.fired.contains_key("half"), "the phase waits for the next round");
    assert!(s.can_advance_round());
    s.advance_round();
    s.step_next();
    assert_eq!(s.phases.fired.get("half").copied(), Some((2, 0)), "the phase fires at the next round's start");
    assert_eq!(s.w.units[BOSS].hp, FLOOR, "no carry into the new phase");
}

#[test]
fn a_round_phase_and_a_fired_last_phase_stop_nothing() {
    // a round phase sets no threshold: the boss takes the whole hit
    let mut s = session(json!([{"id": "r2", "name": "Later", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "line", "text": "Later."}]}]));
    s.w.units[BOSS].hp = FLOOR + 1;
    strike_all(&mut s);
    resolve(&mut s);
    assert!(s.phase_breaks().is_empty() && s.w.units[BOSS].hp < FLOOR, "hp {}", s.w.units[BOSS].hp);
    // the last hp_below phase already fired: nothing stops the hits
    let mut s = session(half_phase(json!([])));
    s.phases.mark_fired("half", 0, 0);
    s.w.units[BOSS].hp = FLOOR + 1;
    strike_all(&mut s);
    resolve(&mut s);
    assert!(s.phase_breaks().is_empty() && s.w.units[BOSS].hp < FLOOR);
}

#[test]
fn a_phase_vignette_adds_its_cue() {
    let mut s = session(half_phase(json!([{"type": "vignette", "id": "boss_turns"}])));
    s.w.units[BOSS].hp = FLOOR + 1;
    strike_all(&mut s);
    resolve(&mut s);
    assert!(s.vignettes().is_empty(), "not before the phase fires");
    s.advance_round();
    s.step_next();
    let cues = s.vignettes().to_vec();
    assert_eq!(cues.len(), 1);
    assert_eq!((cues[0].phase.as_str(), cues[0].vignette.as_str(), cues[0].round, cues[0].block), ("half", "boss_turns", 2, 0));
}
