//! PR6-9 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 5, B6) on a scratch catalog built here (no shipped data): the
//! objective `retreat`. Expectations come from the rule: the floor is max HP x `hp_floor` / 100 rounded down (at least
//! 1, default 25); damage never takes the retreat unit below it (the phase guard), a scripted kill goes past it; once
//! the unit is at or below its floor the block's verdict is a win and the unit leaves the field alive with the fate
//! `withdrawn`. `withdrawn` differs from `fled` (a unit that leaves on its own: another foe withdrawing in the same
//! fight reads fled) and from `killed`; it counts as defeated (pays common rewards).

use indexmap::IndexMap;
use rpgkit_battle::encounter::{floor_hp, Objectives, HP_FLOOR_DEFAULT, RETREAT, WON};
use rpgkit_battle::grid::V2;
use rpgkit_battle::session::{EncounterDef, Fate, Phase, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

/// The roster order: Ash (0), then the guard and the commander.
const GUARD: UnitId = 1;
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

fn retreat(extra: Value) -> Session {
    let mut fields = json!({"objective": RETREAT, "objective_unit": "k_boss"});
    for (k, v) in extra.as_object().unwrap() {
        fields[k] = v.clone();
    }
    session(row(fields))
}

#[test]
fn the_floor_is_the_rounded_down_share_of_max_hp_and_defaults_to_25() {
    assert_eq!(HP_FLOOR_DEFAULT, 25);
    assert_eq!(floor_hp(200, 25), 50);
    assert_eq!(floor_hp(50, 25), 12, "12.5 rounds down");
    assert_eq!(floor_hp(3, 25), 1, "at least 1 HP");
    let s = retreat(json!({}));
    assert_eq!(s.objective.objective, RETREAT);
    assert_eq!(s.objective.hp_floor, HP_FLOOR_DEFAULT);
    assert_eq!(s.w.hp_floor, Some((BOSS, BOSS_HP * HP_FLOOR_DEFAULT / 100)));
    let s = retreat(json!({"hp_floor": 40}));
    assert_eq!(s.w.hp_floor, Some((BOSS, BOSS_HP * 40 / 100)));
    // a rout (the default objective) has no floor; an unknown objective reads as rout
    assert_eq!(session(row(json!({}))).w.hp_floor, None);
    assert_eq!(session(row(json!({"objective": "BOGUS"}))).objective.objective, "rout");
}

#[test]
fn damage_stops_at_the_floor_and_a_scripted_kill_goes_past_it() {
    let mut s = retreat(json!({}));
    let floor = BOSS_HP * HP_FLOOR_DEFAULT / 100;
    s.w.damage(BOSS, 30, false);
    assert_eq!(s.w.units[BOSS].hp, BOSS_HP - 30, "above the floor damage lands as dealt");
    s.w.damage(BOSS, 10 * BOSS_HP, false);
    assert_eq!(s.w.units[BOSS].hp, floor, "a lethal blow stops at the floor (no one-turn kill)");
    assert!(s.w.units[BOSS].is_alive());
    // the guard is not the retreat unit: it falls as usual
    s.w.damage(GUARD, 10_000, false);
    assert!(!s.w.units[GUARD].is_alive());
    let mut s = retreat(json!({}));
    s.w.kill(BOSS, true);
    assert_eq!(s.w.units[BOSS].hp, 0, "a scripted kill past the guards takes it down");
}

#[test]
fn at_the_floor_the_unit_withdraws_alive_and_the_fight_is_won() {
    let mut s = retreat(json!({}));
    s.w.damage(BOSS, 10 * BOSS_HP, false);
    play_round(&mut s);
    assert_eq!(s.outcome, WON, "the retreat clears the room with the guard still standing");
    assert_eq!(s.objective.note, "Commander withdraws.");
    let boss = &s.w.units[BOSS];
    assert!(boss.is_alive() && boss.is_withdrawn && s.w.grid.positions[BOSS].is_none(), "it left the field alive");
    assert!(s.w.units[GUARD].is_alive() && !s.w.units[GUARD].is_withdrawn, "the guard stays");
    let fates = s.fates();
    assert_eq!(fates, vec![("k_guard".to_string(), Fate::Standing), ("k_boss".to_string(), Fate::Withdrawn)]);
    assert_eq!(Fate::Withdrawn.name(), "withdrawn");
    assert!(Fate::Withdrawn.defeated(), "a withdrawn foe pays its common rewards");
    assert_ne!(Fate::Withdrawn, Fate::Killed);
}

#[test]
fn above_the_floor_the_fight_goes_on_and_a_kill_still_reads_killed() {
    let mut s = retreat(json!({}));
    s.w.damage(BOSS, BOSS_HP / 2, false);
    play_round(&mut s);
    assert!(s.outcome.is_empty(), "above the floor the fight goes on: {}", s.outcome);
    let mut s = retreat(json!({}));
    s.w.kill(BOSS, true);
    s.w.kill(GUARD, true);
    play_round(&mut s);
    assert_eq!(s.outcome, WON);
    assert_eq!(s.fates(), vec![("k_guard".to_string(), Fate::Killed), ("k_boss".to_string(), Fate::Killed)], "a scripted kill is killed, not withdrawn");
}

#[test]
fn withdrawn_differs_from_fled() {
    // another foe leaving the field on its own in the same fight is fled; the retreat unit at its floor is withdrawn
    let mut s = retreat(json!({}));
    assert!(s.w.withdraw_in_place(GUARD));
    s.w.damage(BOSS, 10 * BOSS_HP, false);
    play_round(&mut s);
    assert_eq!(s.outcome, WON);
    assert_eq!(s.fates(), vec![("k_guard".to_string(), Fate::Fled), ("k_boss".to_string(), Fate::Withdrawn)]);
    // a slay target that leaves is fled and the fight is not won (the slay rule, unchanged)
    let mut s = session(row(json!({"objective": "slay", "objective_unit": "k_boss"})));
    assert!(s.w.withdraw_in_place(BOSS));
    play_round(&mut s);
    assert_eq!(s.outcome, "fled");
    assert_eq!(s.fates()[1], ("k_boss".to_string(), Fate::Fled));
}

#[test]
fn the_objective_line_names_the_unit_and_the_floor() {
    let o = row(json!({"objective": RETREAT, "objective_unit": "k_boss", "hp_floor": 30}));
    let text = Objectives::text_of(o.as_object().unwrap(), |r| if r == "k_boss" { "Commander".into() } else { r.into() });
    assert_eq!(text, "Drive Commander back to 30% HP, or defeat every enemy.");
}
