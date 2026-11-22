//! PR7-75 (PT-40) on a scratch catalog built here (no shipped data): `Session::end_reason` says why a battle ended:
//! the party's retreat, the round limit, the objective's own note, the last standing heroes leaving the field, every
//! hero down; nothing while it goes on.

use indexmap::IndexMap;
use rpgkit_battle::encounter::{FLED, LOST, RETREAT, WON};
use rpgkit_battle::grid::V2;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

const BOSS: UnitId = 2;
const CELLS: [V2; 3] = [V2 { x: 0, y: 4 }, V2 { x: 11, y: 0 }, V2 { x: 11, y: 8 }];
const BOSS_HP: i32 = 200;

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
    let unit = |id: &str, name: &str, side: &str, hp: i32| json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": hp, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("k_ash", "Ash", "party", 5000), unit("k_guard", "Guard", "enemy", 5000), unit("k_boss", "Commander", "enemy", BOSS_HP)],
    }))
}

/// A fight on an open floor (Ash against a guard and the commander) with `extra` merged over the row.
fn row(extra: Value) -> Value {
    let mut row = json!({"id": "gate", "name": "The Gate", "rows": vec![".".repeat(12); 9], "party": ["k_ash"], "enemies": ["k_guard", "k_boss"],
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 6, "rng_seed": 1337});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

fn session(row: Value) -> Session {
    let def = EncounterDef::from_obj(row.as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in CELLS.into_iter().enumerate() {
        s.w.grid.positions[c] = Some(cell);
    }
    s.replan_enemies();
    s
}

/// Resolves the round with an empty party plan, up to the replay (a verdict ends it early).
fn play_round(s: &mut Session) {
    assert!(s.confirm_plan(), "the plan is confirmed");
    while s.phase != Phase::Replay {
        s.step_next();
    }
}

const ASH: UnitId = 0;

#[test]
fn nothing_while_the_battle_goes_on() {
    let s = session(row(json!({})));
    assert_eq!(s.end_reason(), "");
}

#[test]
fn a_retreat_says_the_party_retreated() {
    let mut s = session(row(json!({})));
    s.retreat().expect("a fled-by-default fight allows it");
    assert_eq!(s.outcome, FLED);
    assert_eq!(s.end_reason(), "The party retreated from the fight.");
}

#[test]
fn the_round_limit_says_so() {
    let mut s = session(row(json!({"max_rounds": 1})));
    play_round(&mut s);
    assert!(s.timed_out(), "the one round ran out");
    assert_eq!(s.end_reason(), "The round limit ran out (1 rounds).");
}

#[test]
fn the_objectives_note_is_the_reason() {
    let mut s = session(row(json!({"objective": RETREAT, "objective_unit": "k_boss"})));
    s.w.damage(BOSS, 10 * BOSS_HP, false);
    play_round(&mut s);
    assert_eq!(s.outcome, WON);
    assert_eq!(s.end_reason(), "Commander withdraws.");
}

#[test]
fn the_last_hero_leaving_the_field_reads_fled_with_the_name() {
    let mut s = session(row(json!({})));
    assert!(s.w.withdraw_in_place(ASH), "Ash leaves the field");
    play_round(&mut s);
    assert_eq!(s.outcome, FLED);
    assert_eq!(s.end_reason(), "No hero was left fighting: Ash left the field.");
}

#[test]
fn every_hero_down_reads_lost() {
    let mut s = session(row(json!({})));
    s.w.kill(ASH, true);
    play_round(&mut s);
    assert_eq!(s.outcome, LOST);
    assert_eq!(s.end_reason(), "Every hero went down.");
}

