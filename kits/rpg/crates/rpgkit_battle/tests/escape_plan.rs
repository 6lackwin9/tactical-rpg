//! ESCAPE-PLAN1 (`PR7-60`, `FB-401`) on scratch catalogs built here (no shipped data). Expectations come from the row:
//! (1) a hand move may end on an exit for a unit Withdraw lets leave, and leaves by it (the holder may not); (2) a pick
//! that would drop another member's hand pick is refused with that member's reason and the battle stays as it was, and
//! a party member's hand move may pass through a friend's tile mid-block but never end a block on one; (3) the hold
//! pick `nearest` names the hero with the shortest walk to a hold tile, and a pick may be changed until round 1 is
//! confirmed; (4) a party member's Withdraw spends no AP it does not have.

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::PlanStep;
use rpgkit_battle::round_intent::RoundTarget;
use rpgkit_battle::round_intent_plan::{self as rip, Pick};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::Kind;
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, AP_COST, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const ASH: UnitId = 0;
const BRYN: UnitId = 1;
const COLE: UnitId = 2;
const DARA: UnitId = 3;
const FOE: UnitId = 4;
const WINCH: V2 = V2 { x: 10, y: 1 };
const EXIT: V2 = V2 { x: 11, y: 4 };

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("ash", "party"), unit("bryn", "party"), unit("cole", "party"), unit("dara", "party"), unit("brute", "enemy")]}))
}

fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST], "basics": ["move", "rest", "strike"]}).as_object().unwrap())
}

/// A four-hero fight on `rows` with exits down the right edge, `extra` merged over the row, under the round intent
/// ruleset with AP costs; the units on `cells` (heroes, then the foe) and the foe's plans dropped.
fn session(rows: Value, extra: Value, cells: [V2; 5]) -> Session {
    let mut row = json!({"id": "esc", "name": "Escape", "rows": rows, "party": ["ash", "bryn", "cole", "dara"], "enemies": ["brute"],
        "exits": [[11, 3], [11, 4], [11, 5]], "objective": "escape", "objective_count": 3,
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 4, "rng_seed": 1337});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = timeline();
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in cells.into_iter().enumerate() {
        s.w.grid.positions[c] = Some(cell);
    }
    let enemies = s.w.enemies.clone();
    s.resolver.scheduled.retain(|a| !enemies.contains(&a.unit));
    s
}

fn floor() -> Value {
    json!(vec![".".repeat(12); 9])
}

fn hold() -> Value {
    json!({"stay_behind": {"mode": "hold", "tiles": [[WINCH.x, WINCH.y]], "tiles_label": "the winch"}})
}

fn mv(s: &Session) -> usize {
    s.w.book().lowest(Kind::Move)
}

fn resolve(s: &mut Session) {
    assert!(s.confirm_plan(), "the plan is confirmed");
    while s.phase != Phase::Replay {
        s.step_next();
    }
}

/// (1) a hand move onto an open exit leaves by it; the holder's is refused, as its Withdraw is.
#[test]
fn a_hand_move_may_end_on_an_exit_and_leaves_by_it() {
    let mut s = session(floor(), hold(), [WINCH, V2::new(9, 4), V2::new(9, 6), V2::new(9, 7), V2::new(0, 8)]);
    s.stay_pick(ASH).unwrap();
    let go = PlanStep::go_to(mv(&s), EXIT);
    assert_eq!(s.plan.add_step(&s.w, BRYN, go.clone(), true), "", "Bryn plans a move onto the exit");
    assert!(s.plan.compile(&s.w, BRYN).withdrawn, "the move leaves by the exit");
    assert_eq!(s.plan.add_step(&s.w, ASH, go, true), "That tile can't be moved to.", "the holder stays");
    resolve(&mut s);
    assert!(s.w.units[BRYN].is_withdrawn, "Bryn walked out through the open gate");
    // the round intent planner takes the same move as a hand pick
    let mut s = session(floor(), hold(), [WINCH, V2::new(9, 4), V2::new(9, 6), V2::new(9, 7), V2::new(0, 8)]);
    s.stay_pick(ASH).unwrap();
    assert_eq!(s.pick(BRYN, RoundTarget::Cell(EXIT), &[Pick::MoveHere(EXIT)]).0, Ok(()));
    assert!(rip::move_tiles(&s.w, &s.plan, COLE).iter().any(|t| t.cell == EXIT), "the board marks the exit as a Move tile");
}

/// (2) a pick that would drop another member's hand pick is refused with its reason; nothing changes.
#[test]
fn a_pick_never_drops_another_members_pick() {
    // a wall across row 4 with one gap at (6, 4)
    let mut rows: Vec<String> = vec![".".repeat(12); 9];
    rows[4] = "######.#####".into();
    // (two move blocks of 2 tiles: Bryn's first block ends in the gap)
    let mut s = session(json!(rows), json!({}), [V2::new(5, 5), V2::new(6, 6), V2::new(0, 0), V2::new(0, 1), V2::new(11, 0)]);
    let to = V2::new(6, 2);
    assert_eq!(s.pick(BRYN, RoundTarget::Cell(to), &[Pick::MoveHere(to)]).0, Ok(()), "Bryn walks through the gap");
    let plan_of = |s: &Session| (rip::picks_of(&s.w, BRYN), s.plan.steps_of(BRYN).len(), s.plan.planned_position(&s.w, BRYN));
    let before = plan_of(&s);
    let gap = V2::new(6, 4);
    let (r, later) = s.pick(ASH, RoundTarget::Cell(gap), &[Pick::MoveHere(gap)]);
    let why = r.expect_err("Ash may not end in the gap Bryn's move ends a block in");
    assert!(why.starts_with("Blocked by bryn's plan"), "{why}");
    assert!(later.is_empty());
    assert_eq!(plan_of(&s), before, "Bryn's plan stays");
    assert!(rip::picks_of(&s.w, ASH).is_empty(), "Ash's pick is not taken");
    assert!(!rip::move_tiles(&s.w, &s.plan, ASH).iter().any(|t| t.cell == gap), "the board does not offer the gap");
}

/// (2) a hero's hand move passes through a friend standing in its way and walks on.
#[test]
fn a_hand_move_passes_through_a_friend() {
    // a one-tile corridor along row 4: Ash stands in it, Bryn walks through Ash in one move block
    let mut rows: Vec<String> = vec!["############".into(); 9];
    rows[4] = "............".into();
    let mut s = session(json!(rows), json!({"exits": []}), [V2::new(3, 4), V2::new(2, 4), V2::new(0, 4), V2::new(11, 4), V2::new(10, 4)]);
    let to = V2::new(4, 4);
    assert_eq!(s.pick(BRYN, RoundTarget::Cell(to), &[Pick::MoveHere(to)]).0, Ok(()), "through Ash's tile");
    resolve(&mut s);
    assert_eq!(s.w.pos(BRYN), to, "Bryn walked through Ash");
    assert_eq!(s.w.pos(ASH), V2::new(3, 4), "Ash did not move");
    // never onto a friend's tile
    let mut s = session(json!(rows), json!({"exits": []}), [V2::new(3, 4), V2::new(2, 4), V2::new(0, 4), V2::new(11, 4), V2::new(10, 4)]);
    assert!(s.pick(BRYN, RoundTarget::Cell(V2::new(3, 4)), &[Pick::MoveHere(V2::new(3, 4))]).0.is_err(), "not onto Ash");
}

/// (3) `nearest` names the hero with the shortest walk to the winch; a pick may change until round 1 is confirmed.
#[test]
fn the_hold_pick_suggests_the_nearest_and_may_change_until_confirm() {
    let extra = json!({"stay_behind": {"mode": "hold", "tiles": [[WINCH.x, WINCH.y]], "tiles_label": "the winch", "pick_default": "nearest"}});
    let mut s = session(floor(), extra, [V2::new(1, 4), V2::new(2, 4), V2::new(8, 2), V2::new(3, 4), V2::new(0, 8)]);
    s.w.units[ASH].hp = 6000;
    assert_eq!(s.stay_pick_suggestion(), Some(COLE), "Cole stands nearest the winch, whoever has the most HP");
    assert_eq!(s.stay_pick(ASH), Ok(()));
    assert_eq!(s.stay_pick(DARA), Ok(()), "a new pick replaces the old one while round 1 is planned");
    assert_eq!(s.w.hold.as_ref().unwrap().holder, Some(DARA));
    assert!(s.stay_pick(FOE).is_err());
    assert!(s.confirm_plan());
    assert!(s.stay_pick(BRYN).is_err(), "no change after round 1 is confirmed");
    // `highest_hp` stays the default
    let mut s = session(floor(), hold(), [V2::new(1, 4), V2::new(2, 4), V2::new(8, 2), V2::new(3, 4), V2::new(0, 8)]);
    s.w.units[ASH].hp = 6000;
    assert_eq!(s.stay_pick_default(), Some(ASH));
}

/// (4) a hero's Withdraw walks only as far as its AP pays for, and is refused with none.
#[test]
fn a_heros_withdraw_spends_no_ap_it_lacks() {
    let mut s = session(floor(), json!({}), [V2::new(2, 4), V2::new(0, 0), V2::new(0, 1), V2::new(0, 2), V2::new(0, 8)]);
    s.w.units[ASH].ap = 3;
    assert_eq!(s.plan.add_step(&s.w, ASH, PlanStep::withdraw(), true), "");
    let st = s.plan.compile(&s.w, ASH);
    assert!(st.ap_left >= 0, "AP left {}", st.ap_left);
    assert!(!st.withdrawn, "three tiles do not reach the exit");
    let mut s = session(floor(), json!({}), [V2::new(2, 4), V2::new(0, 0), V2::new(0, 1), V2::new(0, 2), V2::new(0, 8)]);
    s.w.units[ASH].ap = 0;
    assert_eq!(s.plan.add_step(&s.w, ASH, PlanStep::withdraw(), true), "Not enough AP to withdraw.");
}
