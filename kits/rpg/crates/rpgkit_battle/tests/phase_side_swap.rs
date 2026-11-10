//! PHASEFX1 (docs/specs/ACTS23.md PK6 hooks l and m) on a scratch catalog built here (no shipped data): the phase
//! effects `side` and `swap`. Expectations come from the rules: a `side` to `hostile_all` puts the unit on the enemy
//! side as a rogue whose foes are every other unit (both sides) and whose only friend is itself, the other enemies
//! count it as a foe, the party's lists are unchanged, and the encounter is won only once it falls too; a `side` to
//! `party` makes the unit a guest that no longer counts as a foe (and no longer unkillable, S1 being enemy-side only);
//! a turned unit leaves its morale group; the fates keep the spawned enemy list; the metrics book the hits between a
//! turned unit and its old side as friendly fire and balance; a hero is never turned. A `swap` replaces the unit
//! in place: the same unit id, cell and side, the HP the same share of the new row's max HP (`encounter::swap_hp`,
//! roundi) or full, its statuses gone, the new row's AI profile, its encounter ref (fate, unkillable) kept, one EVENT cell
//! naming the change, the metrics balanced; a swap at a block start cancels the unit's actions not yet landed.

use indexmap::IndexMap;
use rpgkit_battle::encounter::{swap_hp, SWAP_FULL, SWAP_KEEP_PCT};
use rpgkit_battle::grid::V2;
use rpgkit_battle::log::{side_line, Entry, PHASE_SIDE, PHASE_SWAP};
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::session::{EncounterDef, Fate, Phase, Session};
use rpgkit_battle::world::{Role, UnitId};
use rpgkit_data::Catalog;
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

/// The roster order: party, then enemies (brute, two guards, the boss).
const ASH: UnitId = 0;
const BRYN: UnitId = 1;
const BRUTE: UnitId = 2;
const GUARD: UnitId = 3;
const GUARD_B: UnitId = 4;
const BOSS: UnitId = 5;
const ENEMIES: [UnitId; 4] = [BRUTE, GUARD, GUARD_B, BOSS];

/// Heroes on the left edge; the boss beside a guard, far from the heroes; the brute and the other guard apart.
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
    let unit = |id: &str, name: &str, side: &str, hp: i32, role: &str| {
        json!({"id": id, "name": name, "side": side, "stat_mode": "fixed", "hp": hp, "speed": 12, "attack_range": 1, "skill_cap": 5, "ai_role": role})
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("k_ash", "Ash", "party", 5000, "AGGRESSOR"), unit("k_bryn", "Bryn", "party", 5000, "AGGRESSOR"), unit("k_brute", "Brute", "enemy", 5000, "AGGRESSOR"),
            unit("k_guard", "Guard", "enemy", 5000, "AGGRESSOR"), unit("k_boss", "Boss", "enemy", 5000, "AGGRESSOR"), unit("k_beast", "Beast", "enemy", 8000, "SNIPER")],
    }))
}

/// A rout on an open floor with `phases`.
fn row(phases: Value) -> Value {
    row_with(phases, json!({}))
}

/// `row` with `extra` merged over it.
fn row_with(phases: Value, extra: Value) -> Value {
    let mut row = json!({"id": "turn", "name": "The Turn", "rows": vec![".".repeat(12); 9], "party": ["k_ash", "k_bryn"],
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

/// The living units back on CELLS.
fn place(s: &mut Session) {
    for (c, cell) in CELLS.into_iter().enumerate() {
        if s.w.units[c].is_alive() {
            s.w.grid.positions[c] = Some(cell);
        }
    }
}

/// Resolves the round with an empty party plan (until the replay or the end of the fight).
fn resolve(s: &mut Session) {
    assert!(s.confirm_plan(), "the plan is confirmed");
    while s.phase != Phase::Replay && !s.combat_over {
        s.step_next();
    }
}

/// `resolve`, then the units back on CELLS and the next round (round-trigger phases fire at its start).
fn next_round(s: &mut Session) {
    resolve(s);
    assert!(s.can_advance_round(), "the fight goes on");
    place(s);
    s.advance_round();
}

/// A round-2 phase named Turn with `effects`.
fn at_round_two(effects: Value) -> Value {
    json!([{"id": "turn", "name": "Turn", "trigger": {"kind": "round", "round": 2}, "effects": effects}])
}

fn cells<'a>(s: &'a Session, subtype: &str) -> Vec<&'a Entry> {
    s.log.iter().filter(|e| e.category == "EVENT" && e.subtype == subtype).collect()
}

fn sorted(mut v: Vec<UnitId>) -> Vec<UnitId> {
    v.sort();
    v
}

#[test]
fn a_hostile_all_side_fights_both_sides_and_counts_as_a_foe() {
    let mut s = session(row(at_round_two(json!([{"type": "side", "units": ["k_boss", "k_ash"], "to": "hostile_all"}]))));
    // round 1: the two sides as always
    assert!(s.w.sides.rogues().is_empty());
    assert_eq!(s.w.foes_of(BOSS), &[ASH, BRYN]);
    next_round(&mut s);
    assert_eq!(s.phases.fired.get("turn"), Some(&(2, 0)));
    // the boss is a rogue on the enemy side; the hero named beside it is never turned
    assert_eq!(s.w.sides.rogues(), &[BOSS]);
    assert!(s.w.enemies.contains(&BOSS) && !s.w.allies.contains(&BOSS));
    assert!(s.w.allies.contains(&ASH));
    assert_eq!(sorted(s.w.foes_of(BOSS).to_vec()), vec![ASH, BRYN, BRUTE, GUARD, GUARD_B], "every other unit is its foe");
    assert_eq!(s.w.friends_of(BOSS), &[BOSS], "it has no friend but itself");
    for c in [BRUTE, GUARD, GUARD_B] {
        assert_eq!(sorted(s.w.foes_of(c).to_vec()), vec![ASH, BRYN, BOSS], "the other enemies fight it");
        assert!(!s.w.friends_of(c).contains(&BOSS));
    }
    assert_eq!(sorted(s.w.foes_of(ASH).to_vec()), sorted(s.w.enemies.clone()), "the party's foes are unchanged");
    assert_eq!(s.w.friends_of(ASH), s.w.allies.as_slice());
    // one cell naming the change
    let side = cells(&s, PHASE_SIDE);
    assert_eq!(side.len(), 1);
    assert_eq!(side[0].detail, side_line(&["Boss"], "hostile_all"));
    assert_eq!(side[0].detail, "Boss turns on everyone.");
    assert_eq!((side[0].actor.as_str(), side[0].target.as_str(), side[0].target_is_ally), ("The Turn", "Boss", false));
    // the AI picks targets on both sides: the boss and the guard beside it trade blows
    s.replan_enemies();
    resolve(&mut s);
    let hit = |by: &str, on: &str| s.log.iter().any(|e| e.category == "ATTACK" && e.actor == by && e.target == on);
    let guard = s.w.units[GUARD].name.clone();
    assert!(hit("Boss", &guard), "the rogue strikes the guard beside it");
    assert!(hit(&guard, "Boss"), "the guard strikes the rogue");
}

#[test]
fn a_rout_ends_only_when_the_rogue_falls_too() {
    let mut s = session(row(at_round_two(json!([{"type": "side", "units": ["k_boss"], "to": "hostile_all"}]))));
    next_round(&mut s);
    for c in [BRUTE, GUARD, GUARD_B] {
        s.w.kill(c, true);
    }
    resolve(&mut s);
    assert_eq!(s.outcome, "", "the rogue still stands: the fight goes on");
    assert!(s.can_advance_round());
    place(&mut s);
    s.advance_round();
    s.w.kill(BOSS, true);
    resolve(&mut s);
    assert_eq!(s.outcome, "won", "every foe down, the rogue included");
}

#[test]
fn a_side_to_party_makes_a_guest_that_is_no_longer_a_foe() {
    let mut s = session(row(at_round_two(json!([{"type": "side", "units": ["k_boss"], "to": "party"}]))));
    next_round(&mut s);
    assert!(s.w.allies.contains(&BOSS) && !s.w.enemies.contains(&BOSS));
    assert!(s.guests.contains(&BOSS), "planned as a guest");
    assert!(s.w.sides.rogues().is_empty());
    assert_eq!(sorted(s.w.foes_of(BOSS).to_vec()), vec![BRUTE, GUARD, GUARD_B]);
    assert_eq!(cells(&s, PHASE_SIDE)[0].detail, "Boss joins the party side.");
    for c in [BRUTE, GUARD, GUARD_B] {
        s.w.kill(c, true);
    }
    resolve(&mut s);
    assert_eq!(s.outcome, "won", "the turned boss is no foe");
    assert!(s.w.units[BOSS].is_alive());
    // the fates keep the spawned enemy list (each `ref#n` stays the roster's): the turned boss is still there, standing
    let fates = s.fates();
    assert_eq!(fates.iter().map(|(r, _)| r.as_str()).collect::<Vec<_>>(), vec!["k_brute", "k_guard", "k_guard", "k_boss"]);
    assert_eq!(fates[3].1, Fate::Standing);
}

#[test]
fn an_unkillable_enemy_turned_party_is_killable_and_a_rogue_stays_unkillable() {
    for (to, kept) in [("party", false), ("hostile_all", true), ("enemy", true)] {
        let mut s = session(row_with(at_round_two(json!([{"type": "side", "units": ["k_boss"], "to": to}])), json!({"unkillable": ["k_boss"]})));
        assert!(s.w.unkillable.contains(&BOSS));
        next_round(&mut s);
        assert_eq!(s.w.unkillable.contains(&BOSS), kept, "to {to}: S1 is enemy-side only");
    }
}

/// With morale on (one group of the four enemies), a unit turned `party` or `hostile_all` leaves the group: the group
/// shrinks to three, the unit has no group and its authored profile is back; when the group breaks the remaining
/// guard surrenders and the turned boss does not.
#[test]
fn a_turned_unit_leaves_its_morale_group() {
    for to in ["party", "hostile_all"] {
        let mut s = session(row_with(at_round_two(json!([{"type": "side", "units": ["k_boss"], "to": to}])), json!({"morale": true})));
        assert_eq!((s.morale.groups.len(), s.morale.groups[0].size), (1, 4));
        assert_eq!(s.morale_group_of(BOSS), 0);
        next_round(&mut s);
        assert_eq!(s.morale_group_of(BOSS), -1, "to {to}");
        assert_eq!(s.morale.groups[0].size, 3, "to {to}: the group shrinks");
        assert!(!s.morale.groups[0].members.iter().any(|k| k.starts_with("k_boss")), "to {to}");
        assert_eq!(s.morale_state_of(BOSS), "", "to {to}");
        s.w.kill(BRUTE, true);
        s.w.kill(GUARD, true);
        resolve(&mut s);
        assert!(s.morale.groups[0].broken, "to {to}: two of three down breaks the group");
        assert!(s.w.units[GUARD_B].is_withdrawn, "to {to}: the remaining guard surrenders");
        assert!(s.w.units[BOSS].is_active() && !s.w.units[BOSS].is_withdrawn, "to {to}: the turned boss stays");
        assert!(!s.morale.surrendered.iter().any(|k| k.starts_with("k_boss")), "to {to}");
    }
}

/// The metrics with a rogue: the hits between it and its old side are booked as friendly fire (the sides are the
/// fight's start sides) and every balance holds.
#[test]
fn the_metrics_balance_with_a_rogue() {
    let mut s = session(row(at_round_two(json!([{"type": "side", "units": ["k_boss"], "to": "hostile_all"}]))));
    s.attach_metrics(BattleMetrics::new(), true);
    next_round(&mut s);
    s.replan_enemies();
    resolve(&mut s);
    assert!(s.log.iter().any(|e| e.category == "ATTACK" && e.actor == "Boss" && !e.target_is_ally), "the rogue strikes an enemy");
    let mut m = s.metrics.take().unwrap();
    m.finish(&s);
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    assert!(m.units.values().all(|r| r.other_loss == 0 && r.other_gain == 0), "nothing unexplained");
    assert!(m.units.values().any(|r| r.ff_taken > 0), "booked as friendly fire");
}

/// The boss's (HP, max HP) after round 1, wounded to a quarter and empowered (then the metrics follow the fight when
/// `metrics`); then round 2 starts (the swap fires).
fn swap_at_round_two(s: &mut Session, metrics: bool) -> (i32, i32) {
    let hp = s.w.units[BOSS].hp;
    s.w.damage(BOSS, hp - hp / 4, true);
    status::grant_now(&mut s.w.units[BOSS], "empower", 5, 10, "test");
    if metrics {
        s.attach_metrics(BattleMetrics::new(), true);
    }
    resolve(s);
    let before = (s.w.units[BOSS].hp, s.w.units[BOSS].max_hp);
    assert!(s.w.units[BOSS].has_status("empower"));
    place(s);
    s.advance_round();
    before
}

#[test]
fn a_swap_keeps_the_cell_the_side_and_the_hp_share() {
    let mut s = session(row(at_round_two(json!([{"type": "swap", "unit": "k_boss", "to": "k_beast"}]))));
    let (hp, max) = swap_at_round_two(&mut s, true);
    let u = &s.w.units[BOSS];
    assert_eq!((u.name.as_str(), u.catalog_id.as_str()), ("Beast", "k_beast"));
    assert_eq!(s.w.grid.positions[BOSS], Some(CELLS[BOSS]), "the same cell");
    assert!(s.w.enemies.contains(&BOSS) && !s.w.allies.contains(&BOSS), "the same side");
    assert_ne!(u.max_hp, max, "the new row's max HP");
    assert_eq!(u.hp, swap_hp(hp, max, u.max_hp, SWAP_KEEP_PCT));
    assert_eq!(u.hp, (u.max_hp as f64 * hp as f64 / max as f64).round() as i32, "the same share, roundi");
    assert!(!u.has_status("empower"), "statuses end");
    assert_eq!(s.w.profiles[&BOSS].role, Role::Sniper, "the new row's AI");
    assert_eq!(s.units_of_ref("k_boss"), vec![BOSS], "it keeps its encounter ref");
    assert!(!s.w.unkillable.contains(&BOSS), "not unkillable before, not after");
    // one cell naming the change, after the phase's own
    let swap = cells(&s, PHASE_SWAP);
    assert_eq!(swap.len(), 1);
    assert_eq!(swap[0].detail, "Boss becomes Beast.");
    assert_eq!((swap[0].actor.as_str(), swap[0].target.as_str(), swap[0].value), ("The Turn", "Beast", u.hp));
    assert!(s.events.contains(&swap[0].detail));
    // the fight plays on with the new unit; the metrics balance (the change is its scripted HP)
    s.replan_enemies();
    resolve(&mut s);
    let mut m = s.metrics.take().unwrap();
    m.finish(&s);
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    assert!(m.units.values().all(|r| r.other_loss == 0 && r.other_gain == 0), "nothing unexplained");
    assert_eq!(m.scripted.get("Boss").copied(), Some(hp - s.log.iter().find(|e| e.subtype == PHASE_SWAP).unwrap().value), "the record keeps the first name");
}

#[test]
fn a_swapped_unit_falls_as_its_old_ref() {
    let mut s = session(row(at_round_two(json!([{"type": "swap", "unit": "k_boss", "to": "k_beast"}]))));
    next_round(&mut s);
    assert_eq!(s.w.units[BOSS].name, "Beast");
    for c in ENEMIES {
        s.w.kill(c, true);
    }
    resolve(&mut s);
    assert_eq!(s.outcome, "won");
    let fates = s.fates();
    assert_eq!(fates.iter().filter(|(r, _)| r == "k_boss").map(|(_, f)| *f).collect::<Vec<_>>(), vec![Fate::Killed]);
    assert!(!fates.iter().any(|(r, _)| r == "k_beast"));
}

#[test]
fn a_swapped_unkillable_unit_stays_unkillable() {
    let mut s = session(row_with(at_round_two(json!([{"type": "swap", "unit": "k_boss", "to": "k_beast"}])), json!({"unkillable": ["k_boss"]})));
    next_round(&mut s);
    assert_eq!(s.w.units[BOSS].name, "Beast");
    assert!(s.w.unkillable.contains(&BOSS), "unkillable follows the encounter ref");
}

#[test]
fn a_full_swap_starts_at_full_hp() {
    let mut s = session(row(at_round_two(json!([{"type": "swap", "unit": "k_boss", "to": "k_beast", "hp": "full"}]))));
    let (hp, max) = swap_at_round_two(&mut s, false);
    assert!(hp < max, "the boss was wounded");
    let u = &s.w.units[BOSS];
    assert_eq!(u.hp, u.max_hp);
    assert_eq!(u.hp, swap_hp(hp, max, u.max_hp, SWAP_FULL));
}

#[test]
fn a_swap_at_a_block_start_cancels_the_units_pending_actions() {
    let phases = json!([{"id": "turn", "name": "Turn", "trigger": {"kind": "round", "round": 2, "block": 3}, "effects": [{"type": "swap", "unit": "k_boss", "to": "k_beast"}]}]);
    let mut s = session(row(phases));
    next_round(&mut s);
    s.replan_enemies();
    assert!(s.confirm_plan());
    while !s.phases.fired.contains_key("turn") && s.phase != Phase::Replay {
        s.step_next();
    }
    assert_eq!(s.phases.fired.get("turn"), Some(&(2, 3)));
    assert_eq!(s.w.units[BOSS].name, "Beast");
    assert!(s.resolver.scheduled.iter().filter(|a| a.unit == BOSS).all(|a| a.landed || a.cancelled), "nothing of the old row is left to land");
}

#[test]
fn encounters_without_the_effects_keep_the_two_sides() {
    let s = session(row(json!([])));
    assert!(s.w.sides.rogues().is_empty());
    for c in ENEMIES {
        assert_eq!(s.w.foes_of(c), s.w.allies.as_slice());
        assert_eq!(s.w.friends_of(c), s.w.enemies.as_slice());
    }
}
