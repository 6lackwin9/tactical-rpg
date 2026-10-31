//! INPUT1-M (docs/specs/INPUT1.md 11, INPUT1_TASKS 3) on a scratch catalog built here (no shipped data): the Maneuver
//! rework under the rule `round_intent`. Expectations come from the rules: MR2's position table for both facings of the
//! table and its rotations, MR3's open tiles and `melee_diagonal` none, MR4's tiers and the worked examples of 11.3,
//! MR5's "Flank protected", MR6's follow-ups, MR7's out-of-range move, MR8's run time (the dash on the start block, the
//! chase budget N + 1 and its 1 AP, the protected miss with no hit), MR9's four costs, MR10's diagonal melee and the
//! riposte, MR1's ranged skill with a position and a Maneuver skill without one, MR14's Disengage retreat and AI row 4,
//! MR16's chase for a ranged attack and a heal (not past root or a hidden target, never a plan move), the `dash_tiles`
//! transition and the dash board event (MR12).

use indexmap::IndexMap;
use rpgkit_battle::command::{Carry, CmdKind, MoveKind};
use rpgkit_battle::grid::V2;
use rpgkit_battle::path::Cells;
use rpgkit_battle::plan::{PlanEntry, PlanStep};
use rpgkit_battle::round_intent::{self, ManeuverKind, Opener, RoundIntent, RoundTarget, FLANK_PROTECTED};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, Position};
use rpgkit_rules::ruleset::{FlankCost, MeleeDiagonal, Ruleset, AP_COST, FLANK, ROUND_INTENT, TIMELINE};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
const BASICS: [&str; 4] = ["move", "rest", "strike", "flank_cut"];

/// Engine skills and the Maneuver kit: Flank Cut (flank, range 2), Hamstring Cut (behind, 3) and Slip Strike (flank, 2,
/// two blocks) as INPUT1-MA writes them; a 4 AP flank skill for MR9; a ranged skill with a position (range 4); a
/// Maneuver attack without a position; the transition's row (`dash_tiles` 2, no position); a ranged shot and a heal.
fn catalog() -> Catalog {
    let atk = |id: &str, intent: &str, blocks: i32, ap: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": 1, "blocks": blocks, "ap_cost": ap, "power_mult": 0.8, "target_mode": "ENEMY"});
        for (k, v) in extra.as_object().unwrap() {
            row[k] = v.clone();
        }
        row
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        atk("flank_cut", "maneuver", 1, 2, json!({"position": "flank", "reach_max": 2})),
        atk("tl_hamstring_cut", "maneuver", 1, 2, json!({"position": "behind", "reach_max": 3})),
        atk("tl_slip_strike", "maneuver", 2, 2, json!({"position": "flank", "reach_max": 2})),
        atk("t_heavy_flank", "maneuver", 2, 4, json!({"position": "flank", "reach_max": 2})),
        atk("t_dash_shot", "maneuver", 1, 2, json!({"position": "flank", "reach_max": 4})),
        atk("t_poke", "maneuver", 1, 1, json!({})),
        atk("t_transition", "maneuver", 1, 2, json!({"dash_tiles": 2})),
        atk("tl_aimed_shot", "assault", 2, 2, json!({"power_mult": 1.1})),
        json!({"id": "tl_field_dressing", "name": "Field Dressing", "kind": "SUPPORT", "round_intent": "support", "level": 1, "blocks": 2, "ap_cost": 2, "heal_base": 8, "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF"}),
    ];
    let unit = |id: &str, side: &str, range: i32, order: Value| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": range, "skill_cap": 5, "deck": {"order": order}});
    let units = vec![
        unit("tl_vanguard", "party", 1, json!({"assault": ["strike"]})),
        unit("tl_spiker", "party", 1, json!({"assault": ["strike"], "maneuver": ["flank_cut", "tl_slip_strike", "tl_hamstring_cut", "t_heavy_flank", "t_poke", "t_transition"]})),
        unit("tl_archer", "party", 3, json!({"assault": ["tl_aimed_shot"], "support": ["tl_field_dressing"]})),
        unit("t_longbow", "party", 5, json!({"maneuver": ["t_dash_shot"]})),
        unit("t_plain", "party", 1, json!({})),
        unit("tl_footman", "enemy", 1, json!({})),
        unit("tl_footman_b", "enemy", 1, json!({})),
    ];
    Catalog::from_value(&json!({"skills": skills, "units": units}))
}

const VANGUARD: UnitId = 0;
const SPIKER: UnitId = 1;
const ARCHER: UnitId = 2;
const LONGBOW: UnitId = 3;
const PLAIN: UnitId = 4;
/// The worked examples' Wolf.
const WOLF: UnitId = 5;
const GOBLIN: UnitId = 6;

/// A timeline ruleset with the round intent, `ap_cost` and (when `flank`) the flank bonus; `melee_diagonal` and the
/// protected-flank cost as given.
fn ruleset(diag: MeleeDiagonal, cost: FlankCost, flank: bool) -> Ruleset {
    let rules: Vec<&str> = if flank { vec![ROUND_INTENT, AP_COST, FLANK] } else { vec![ROUND_INTENT, AP_COST] };
    let mut r = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": BASICS}).as_object().unwrap());
    r.numbers.melee_diagonal = diag;
    r.numbers.flank_protected = cost;
    r
}

fn timeline() -> Ruleset {
    ruleset(MeleeDiagonal::Maneuver, FlankCost::Full, true)
}

/// Every unit parked in a corner or along the edges (tests place the ones they use); the enemies' round-1 plans are
/// dropped so only what a test plans acts.
fn session(r: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["tl_vanguard", "tl_spiker", "tl_archer", "t_longbow", "t_plain"],
        "enemies": ["tl_footman", "tl_footman_b"], "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = r;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    let parked = [V2::new(0, 0), V2::new(0, 8), V2::new(2, 0), V2::new(2, 8), V2::new(4, 0), V2::new(11, 0), V2::new(11, 8)];
    for (c, cell) in parked.into_iter().enumerate() {
        place(&mut s.w, c, cell);
    }
    drop_enemy_plans(&mut s);
    s
}

fn drop_enemy_plans(s: &mut Session) {
    let enemies = s.w.enemies.clone();
    s.resolver.scheduled.retain(|a| !enemies.contains(&a.unit));
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The worked examples' board: Wolf on (6, 4) engaged with tl_vanguard on (7, 4), so Wolf faces right.
fn wolf_board(s: &mut Session) {
    place(&mut s.w, WOLF, V2::new(6, 4));
    place(&mut s.w, VANGUARD, V2::new(7, 4));
    round_intent::set_intent(&mut s.w, WOLF, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut s.w, WOLF, RoundTarget::Unit(VANGUARD)).unwrap();
    assert_eq!(round_intent::facing(&s.w, WOLF), V2::new(1, 0), "Wolf faces right");
    assert_eq!(round_intent::engaged_with(&s.w, WOLF), Some(VANGUARD));
}

/// `c` takes Maneuver Engage on Wolf.
fn engage(s: &mut Session, c: UnitId) {
    round_intent::set_intent(&mut s.w, c, RoundIntent::Maneuver).unwrap();
    round_intent::set_target(&mut s.w, c, RoundTarget::Unit(WOLF)).unwrap();
    round_intent::set_maneuver(&mut s.w, c, ManeuverKind::Engage).unwrap();
}

/// Adds `skill` on Wolf for `c` by hand (no follow-ups); the planner's problem ("" = placed).
fn add(s: &mut Session, c: UnitId, skill: &str) -> String {
    let k = sid(&s.w, skill);
    s.plan.add_step(&s.w, c, PlanStep::engage(k, WOLF), false)
}

fn opener(s: &mut Session, c: UnitId, skill: &str) {
    let k = sid(&s.w, skill);
    rip::set_opener(&mut s.w, c, Some(Opener::Skill(k))).unwrap();
}

fn entry_of(s: &Session, c: UnitId, skill: &str) -> PlanEntry {
    let k = sid(&s.w, skill);
    s.plan.entries_for(&s.w, c).into_iter().find(|e| e.skill == k).unwrap_or_else(|| panic!("{skill} planned"))
}

/// Confirms the plan and resolves the round (enemies plan nothing: their round-1 plans were dropped).
fn resolve(s: &mut Session) {
    assert!(s.confirm_plan());
    drop_enemy_plans(s);
    while s.phase != Phase::Replay {
        s.step_next();
    }
}

fn logged(s: &Session, start: &str) -> Vec<String> {
    s.resolver.log.iter().filter(|l| l.starts_with(start)).cloned().collect()
}

fn sorted(mut v: Vec<V2>) -> Vec<V2> {
    v.sort_by_key(|c| (c.x, c.y));
    v
}

/// MR2's table rotated a quarter turn `n` times (offsets and facing alike).
fn rot(v: V2, n: usize) -> V2 {
    (0..n).fold(v, |v, _| V2::new(-v.y, v.x))
}

/// MR2: for both facings of the table and their rotations (the other six facings), `behind` prefers the tile straight
/// behind, `flank` the two beside, both fall back to the two diagonals between; nothing in the target's front (k 3, 4:
/// the tiles with a positive dot product with the facing) is ever a position. With Wolf facing (1, 1) a behind skill
/// prefers t + (-1, -1).
#[test]
fn positions_follow_the_facing_table() {
    let mut s = session(timeline());
    let t = V2::new(6, 4);
    place(&mut s.w, WOLF, t);
    // (f, behind, diagonals, flanks) as the table lists them
    let table = [
        (V2::new(1, 0), vec![V2::new(-1, 0)], vec![V2::new(-1, -1), V2::new(-1, 1)], vec![V2::new(0, -1), V2::new(0, 1)]),
        (V2::new(1, 1), vec![V2::new(-1, -1)], vec![V2::new(-1, 0), V2::new(0, -1)], vec![V2::new(-1, 1), V2::new(1, -1)]),
    ];
    let none = Cells::new();
    let mut facings = Vec::new();
    for (f, behind, diags, flanks) in &table {
        for n in 0..4 {
            let f = rot(*f, n);
            facings.push(f);
            s.w.round_states.entry(WOLF).or_default().facing = f;
            let at = |offs: &Vec<V2>| sorted(offs.iter().map(|o| t + rot(*o, n)).collect());
            let b = round_intent::positions(&s.w, WOLF, SPIKER, Position::Behind, V2::new(0, 0), true, &none);
            let fl = round_intent::positions(&s.w, WOLF, SPIKER, Position::Flank, V2::new(0, 0), true, &none);
            assert_eq!(sorted(b.preferred.clone()), at(behind), "behind, facing {f}");
            assert_eq!(sorted(fl.preferred.clone()), at(flanks), "flank, facing {f}");
            assert_eq!(sorted(b.fallback.clone()), at(diags), "behind's fallback, facing {f}");
            assert_eq!(sorted(fl.fallback.clone()), at(diags), "flank's fallback, facing {f}");
            for cell in b.preferred.iter().chain(&b.fallback).chain(&fl.preferred).chain(&fl.fallback) {
                let d = *cell - t;
                assert!(d.x * f.x + d.y * f.y <= 0, "{cell} lies in the front of a unit facing {f}");
            }
        }
    }
    assert_eq!(sorted(facings).len(), 8, "every facing once");
    s.w.round_states.entry(WOLF).or_default().facing = V2::new(1, 1);
    assert_eq!(round_intent::positions(&s.w, WOLF, SPIKER, Position::Behind, V2::new(0, 0), true, &none).preferred, [t + V2::new(-1, -1)]);
}

/// MR3: a tile is open with no unit but the attacker on it and not claimed; without diagonal melee for the skill no
/// diagonal neighbour is a position, so with f = (1, 1) behind and both flanks drop and the k = 1 tiles stand alone; a
/// target with no facing faces the attacker.
#[test]
fn open_tiles_claims_and_no_diagonal_melee() {
    let mut s = session(timeline());
    let t = V2::new(6, 4);
    place(&mut s.w, WOLF, t);
    s.w.round_states.entry(WOLF).or_default().facing = V2::new(1, 0);
    let none = Cells::new();
    place(&mut s.w, GOBLIN, V2::new(6, 3));
    let fl = round_intent::positions(&s.w, WOLF, SPIKER, Position::Flank, V2::new(0, 0), true, &none);
    assert_eq!(fl.preferred, [V2::new(6, 5)], "a unit on (6, 3) takes that flank");
    place(&mut s.w, SPIKER, V2::new(6, 5));
    let fl = round_intent::positions(&s.w, WOLF, SPIKER, Position::Flank, V2::new(6, 5), true, &none);
    assert_eq!(fl.preferred, [V2::new(6, 5)], "the attacker's own tile is open");
    let claimed: Cells = [V2::new(6, 5), V2::new(5, 3)].into_iter().collect();
    let fl = round_intent::positions(&s.w, WOLF, SPIKER, Position::Flank, V2::new(0, 0), true, &claimed);
    assert_eq!((fl.preferred.clone(), fl.fallback.clone()), (Vec::new(), vec![V2::new(5, 5)]), "claims are taken");
    place(&mut s.w, GOBLIN, V2::new(11, 8));
    s.w.round_states.entry(WOLF).or_default().facing = V2::new(1, 1);
    for p in [Position::Behind, Position::Flank] {
        let o = round_intent::positions(&s.w, WOLF, ARCHER, p, V2::new(0, 0), false, &none);
        assert!(o.preferred.is_empty(), "{p:?}: the preferred tiles are diagonals");
        assert_eq!(sorted(o.fallback), sorted(vec![t + V2::new(-1, 0), t + V2::new(0, -1)]), "{p:?}: the k = 1 tiles alone");
    }
    s.w.round_states.entry(WOLF).or_default().facing = V2::new(0, 0);
    let o = round_intent::positions(&s.w, WOLF, ARCHER, Position::Behind, V2::new(9, 4), true, &none);
    assert_eq!(o.preferred, [V2::new(5, 4)], "no facing: Wolf faces the attacker on (9, 4), so behind is (5, 4)");
}

/// Section 11.3's worked examples (Wolf on (6, 4) faces right, engaged with tl_vanguard on (7, 4); `melee_diagonal`
/// maneuver), each planned by hand from the stated tile.
#[test]
fn the_worked_examples() {
    let fresh = |from: V2| {
        let mut s = session(timeline());
        wolf_board(&mut s);
        place(&mut s.w, SPIKER, from);
        engage(&mut s, SPIKER);
        s
    };
    // Hamstring Cut (behind, 3) from (3, 4), open: dash (4, 4), (5, 4), cost 2; a hit from behind is a flank (MV3)
    let mut s = fresh(V2::new(3, 4));
    assert_eq!(add(&mut s, SPIKER, "tl_hamstring_cut"), "");
    let e = entry_of(&s, SPIKER, "tl_hamstring_cut");
    assert_eq!((e.pos_before, e.pos_after, e.path.len(), e.carry), (V2::new(3, 4), V2::new(5, 4), 2, Carry::Engage(V2::new(5, 4))));
    assert!(round_intent::flank_bonus_from(&s.w, SPIKER, e.pos_after, WOLF), "behind an engaged Wolf is a flank");
    // a goblin on (5, 4): (5, 3) or (5, 5), 3 tiles each, PL4 ranks
    let mut s = fresh(V2::new(3, 4));
    place(&mut s.w, GOBLIN, V2::new(5, 4));
    assert_eq!(add(&mut s, SPIKER, "tl_hamstring_cut"), "");
    let e = entry_of(&s, SPIKER, "tl_hamstring_cut");
    assert!([V2::new(5, 3), V2::new(5, 5)].contains(&e.pos_after) && e.path.len() == 3, "{e:?}");
    // units on (5, 4), (5, 3), (5, 5): the flank tiles are not behind's fallback, so the flank is protected (MR5)
    let mut s = fresh(V2::new(3, 4));
    for (c, cell) in [(GOBLIN, V2::new(5, 4)), (PLAIN, V2::new(5, 3)), (ARCHER, V2::new(5, 5))] {
        place(&mut s.w, c, cell);
    }
    assert_eq!(add(&mut s, SPIKER, "tl_hamstring_cut"), FLANK_PROTECTED);
    assert!(round_intent::flank_protected(&s.w, WOLF, SPIKER, &s.w.cat.skills[sid(&s.w, "tl_hamstring_cut")], V2::new(3, 4), &Cells::new()));
    opener(&mut s, SPIKER, "tl_hamstring_cut");
    assert_eq!(rip::preview(&s.w, &s.plan, SPIKER, RoundIntent::Maneuver, Some(ManeuverKind::Engage)).err().as_deref(), Some(FLANK_PROTECTED), "the row reads Flank protected");
    // Flank Cut (flank, 2) from (5, 2), open: dash to (6, 3), cost 2; a side hit
    let mut s = fresh(V2::new(5, 2));
    assert_eq!(add(&mut s, SPIKER, "flank_cut"), "");
    let e = entry_of(&s, SPIKER, "flank_cut");
    assert_eq!((e.pos_after, e.path.len()), (V2::new(6, 3), 2));
    // a unit on (6, 3): (6, 5) is 4 away, so the fallback (5, 3), 1 tile (Q11 a), a diagonal hit
    let mut s = fresh(V2::new(5, 2));
    place(&mut s.w, GOBLIN, V2::new(6, 3));
    assert_eq!(add(&mut s, SPIKER, "flank_cut"), "");
    let e = entry_of(&s, SPIKER, "flank_cut");
    assert_eq!((e.pos_after, e.path.len()), (V2::new(5, 3), 1));
    // from (2, 4), open: out of range (the nearest position tile is 4 away): a plain move first (1 AP a tile, at most 2
    // move blocks) to the PL4 tile from which a preferred tile is in range, then the dash to (6, 3). (The example's
    // "2 tiles" cannot reach a tile 2 from (6, 3): (4, 3), its own "such as", is 3 tiles from (2, 4).)
    let mut s = fresh(V2::new(2, 4));
    assert_eq!(add(&mut s, SPIKER, "flank_cut"), "");
    let entries = s.plan.entries_for(&s.w, SPIKER);
    let moves: Vec<&PlanEntry> = entries.iter().filter(|e| skill::kind(&s.w.cat.skills[e.skill]) == Kind::Move).collect();
    let tiles: i32 = moves.iter().map(|e| e.path.len() as i32).sum();
    let ap: i32 = moves.iter().map(|e| e.total_ap()).sum();
    assert!(tiles > 0 && ap == tiles && moves.len() <= 2, "the plain move: {tiles} tiles at 1 AP a tile ({ap} AP) in at most 2 blocks");
    let e = entry_of(&s, SPIKER, "flank_cut");
    assert_eq!((e.pos_before, e.pos_after), (moves.last().unwrap().pos_after, V2::new(6, 3)));
    assert!(e.path.len() <= 2, "the dash fits the range");
    // the nearest tiles within 2 of (6, 3) on the way, (4, 3) and (5, 4), are 3 tiles from (2, 4)
    assert_eq!(tiles, 3, "the shortest move from which a preferred tile is in range");
}

/// MR6: a later skill with a position moves only when the unit's tile is not a position tile: Flank Cut dashes to
/// (6, 3), the follow-ups after it hit from there.
#[test]
fn a_follow_up_on_a_position_tile_stays() {
    let mut s = session(timeline());
    wolf_board(&mut s);
    place(&mut s.w, SPIKER, V2::new(5, 2));
    engage(&mut s, SPIKER);
    opener(&mut s, SPIKER, "flank_cut");
    rip::plan_unit(&mut s.w, &mut s.plan, SPIKER).unwrap();
    let carried: Vec<PlanEntry> = s.plan.entries_for(&s.w, SPIKER).into_iter().filter(|e| matches!(e.carry, Carry::Engage(_))).collect();
    assert!(carried.len() >= 2, "an opener and a follow-up: {carried:?}");
    assert_eq!((carried[0].pos_before, carried[0].pos_after), (V2::new(5, 2), V2::new(6, 3)), "the opener dashes");
    for e in &carried[1..] {
        assert_eq!((e.pos_before, e.pos_after), (V2::new(6, 3), V2::new(6, 3)), "a follow-up hits from the flank tile");
    }
}

/// MR8 c, MR9: the position tiles filled between planning and the start: `[Flank Protected]` at the start with the AP
/// each cost pays of a 4 AP skill (full 4, half 2, fixed 1, none 0), no move and no hit at the impact (no damage, no
/// hit, no riposte); the command keeps its blocks (its wind-up runs to its impact block, not cancelled).
#[test]
fn a_protected_flank_misses_with_its_ap() {
    for (cost, pays) in [(FlankCost::Full, 4), (FlankCost::Half, 2), (FlankCost::Fixed(FlankCost::FIXED_AP), 1), (FlankCost::None, 0)] {
        let mut s = session(ruleset(MeleeDiagonal::Maneuver, cost, true));
        wolf_board(&mut s);
        place(&mut s.w, SPIKER, V2::new(5, 2));
        engage(&mut s, SPIKER);
        assert_eq!(add(&mut s, SPIKER, "t_heavy_flank"), "");
        assert_eq!(entry_of(&s, SPIKER, "t_heavy_flank").total_ap(), 4);
        let hp = s.w.units[WOLF].hp;
        assert!(s.confirm_plan());
        drop_enemy_plans(&mut s);
        for (c, cell) in [(GOBLIN, V2::new(6, 3)), (PLAIN, V2::new(6, 5)), (ARCHER, V2::new(5, 3)), (LONGBOW, V2::new(5, 5))] {
            place(&mut s.w, c, cell);
        }
        // up to the start (block 1)
        while !s.resolver.scheduled.iter().any(|a| a.unit == SPIKER && a.command.missed) && s.phase != Phase::Replay {
            s.step_next();
        }
        let line = format!("[Flank Protected] tl_spiker finds tl_footman's flank protected: t_heavy_flank misses ({pays} AP spent)");
        assert_eq!(logged(&s, "[Flank Protected] tl_spiker finds"), [line], "{cost:?}");
        assert_eq!(s.w.units[SPIKER].ap, UNIT_AP - pays, "{cost:?}");
        assert_eq!(s.w.pos(SPIKER), V2::new(5, 2), "no move");
        let a = s.resolver.scheduled.iter().find(|a| a.unit == SPIKER).unwrap();
        assert!(!a.cancelled && !a.landed && a.impact == 2 && a.command.missed, "its wind-up runs to its impact");
        while s.phase != Phase::Replay {
            s.step_next();
        }
        let a = s.resolver.scheduled.iter().find(|a| a.unit == SPIKER).unwrap();
        assert!(a.landed && a.command.hits.is_empty() && a.command.reactions.is_empty() && a.command.struck.is_empty(), "{cost:?}: no hit");
        assert_eq!(s.w.units[WOLF].hp, hp, "{cost:?}: no damage");
        assert!(a.command.log.starts_with("[Flank Protected]"), "{}", a.command.log);
    }
}

/// MR8 a, MR16, MR12: Wolf (and tl_vanguard) step 1 tile before the start: the position tiles are read around Wolf's
/// tile then with the budget N + 1, the extra tile at the chase's 1 AP; the dash lands at the start as a dash board
/// event (not a walk), and the hit lands at the impact. No plan entry is a move (the dash is not the round's move).
#[test]
fn a_target_that_stepped_is_chased_inside_the_dash() {
    let mut s = session(timeline());
    wolf_board(&mut s);
    place(&mut s.w, SPIKER, V2::new(5, 2));
    engage(&mut s, SPIKER);
    assert_eq!(add(&mut s, SPIKER, "flank_cut"), "");
    assert!(s.plan.entries_for(&s.w, SPIKER).iter().all(|e| skill::kind(&s.w.cat.skills[e.skill]) != Kind::Move));
    let hp = s.w.units[WOLF].hp;
    assert!(s.confirm_plan());
    drop_enemy_plans(&mut s);
    place(&mut s.w, WOLF, V2::new(6, 5));
    place(&mut s.w, VANGUARD, V2::new(7, 5));
    while s.phase != Phase::Replay {
        s.step_next();
    }
    // the flank (6, 4) is 3 tiles from (5, 2): range 2 plus the chase's tile
    assert_eq!(s.w.pos(SPIKER), V2::new(6, 4));
    assert_eq!(logged(&s, "[Dash]"), ["[Dash] tl_spiker dashes at tl_footman: (5, 2) to (6, 4) (chase 1 AP)"]);
    assert_eq!(s.w.units[SPIKER].ap, UNIT_AP - 2 - 1, "the skill's 2 AP and the chase's 1");
    let a = s.resolver.scheduled.iter().find(|a| a.unit == SPIKER).unwrap();
    assert_eq!(a.command.moved, Some(MoveKind::Dash), "the dash board event");
    assert!(s.w.units[WOLF].hp < hp, "the hit lands");
}

/// MR8 b: open position tiles, none within the budget: no move (`[Dash Blocked]`), and the hit lands only when the
/// target is adjacent.
#[test]
fn a_dash_that_cannot_reach_hits_only_when_adjacent() {
    let mut s = session(timeline());
    wolf_board(&mut s);
    place(&mut s.w, SPIKER, V2::new(5, 2));
    engage(&mut s, SPIKER);
    assert_eq!(add(&mut s, SPIKER, "flank_cut"), "");
    let hp = s.w.units[WOLF].hp;
    assert!(s.confirm_plan());
    drop_enemy_plans(&mut s);
    // Wolf and tl_vanguard step 3 tiles down: every position tile is beyond 2 + 1
    place(&mut s.w, WOLF, V2::new(6, 7));
    place(&mut s.w, VANGUARD, V2::new(7, 7));
    while s.phase != Phase::Replay {
        s.step_next();
    }
    assert_eq!(s.w.pos(SPIKER), V2::new(5, 2));
    assert_eq!(logged(&s, "[Dash Blocked]"), ["[Dash Blocked] tl_spiker can't reach tl_footman's flank"]);
    assert_eq!(s.w.units[WOLF].hp, hp, "not adjacent: no hit");
}

/// MR10: a Strike from (5, 3) on Wolf (a diagonal neighbour) does not reach under `maneuver` (the planner walks first)
/// and does under `all` (no move), where a riposte also answers it and Wolf counts as engaged with the striker (MV2);
/// under `maneuver` a diagonal Flank Cut draws no riposte (the counter is not a Maneuver skill). No flank rule here, so
/// no hit is a flank.
#[test]
fn diagonal_melee_by_ruleset_value() {
    let strike_from_diagonal = |diag: MeleeDiagonal| {
        let mut s = session(ruleset(diag, FlankCost::Full, false));
        place(&mut s.w, WOLF, V2::new(6, 4));
        place(&mut s.w, SPIKER, V2::new(5, 3));
        round_intent::set_intent(&mut s.w, WOLF, RoundIntent::Assault).unwrap();
        round_intent::set_target(&mut s.w, WOLF, RoundTarget::Unit(SPIKER)).unwrap();
        round_intent::set_intent(&mut s.w, SPIKER, RoundIntent::Assault).unwrap();
        round_intent::set_target(&mut s.w, SPIKER, RoundTarget::Unit(WOLF)).unwrap();
        assert_eq!(add(&mut s, SPIKER, "strike"), "");
        s
    };
    let s = strike_from_diagonal(MeleeDiagonal::Maneuver);
    assert!(s.plan.entries_for(&s.w, SPIKER).iter().any(|e| skill::kind(&s.w.cat.skills[e.skill]) == Kind::Move), "maneuver: the Strike walks into reach");
    assert_eq!(round_intent::engaged_with(&s.w, WOLF), None, "maneuver: a diagonal neighbour is not in the melee band");
    let mut s = strike_from_diagonal(MeleeDiagonal::All);
    assert!(s.plan.entries_for(&s.w, SPIKER).iter().all(|e| skill::kind(&s.w.cat.skills[e.skill]) != Kind::Move), "all: in reach from the diagonal");
    assert_eq!(round_intent::engaged_with(&s.w, WOLF), Some(SPIKER), "all: MV2 counts the diagonal");
    status::grant_now(&mut s.w.units[WOLF], "riposte", 2, 0, "test");
    let hp = s.w.units[WOLF].hp;
    resolve(&mut s);
    let a = s.resolver.scheduled.iter().find(|a| a.unit == SPIKER).unwrap();
    assert!(s.w.units[WOLF].hp < hp, "all: the Strike lands");
    assert_eq!(a.command.reactions.len(), 1, "all: the riposte answers the diagonal hit");
    // maneuver: Flank Cut from the diagonal (both flanks taken) lands; the riposte does not answer it
    let mut s = session(ruleset(MeleeDiagonal::Maneuver, FlankCost::Full, false));
    wolf_board(&mut s);
    place(&mut s.w, SPIKER, V2::new(5, 2));
    place(&mut s.w, GOBLIN, V2::new(6, 3));
    place(&mut s.w, PLAIN, V2::new(6, 5));
    engage(&mut s, SPIKER);
    assert_eq!(add(&mut s, SPIKER, "flank_cut"), "");
    assert_eq!(entry_of(&s, SPIKER, "flank_cut").pos_after, V2::new(5, 3), "the diagonal fallback");
    status::grant_now(&mut s.w.units[WOLF], "riposte", 2, 0, "test");
    let hp = s.w.units[WOLF].hp;
    resolve(&mut s);
    let a = s.resolver.scheduled.iter().find(|a| a.unit == SPIKER).unwrap();
    assert!(s.w.units[WOLF].hp < hp, "the diagonal Flank Cut lands");
    assert!(a.command.reactions.is_empty(), "no riposte under maneuver");
    // none: Flank Cut never ends on a diagonal of Wolf
    let mut s = session(ruleset(MeleeDiagonal::None, FlankCost::Full, true));
    wolf_board(&mut s);
    place(&mut s.w, SPIKER, V2::new(5, 2));
    place(&mut s.w, GOBLIN, V2::new(6, 3));
    engage(&mut s, SPIKER);
    assert_eq!(add(&mut s, SPIKER, "flank_cut"), "");
    let to = entry_of(&s, SPIKER, "flank_cut").pos_after - V2::new(6, 4);
    assert!(to.x == 0 || to.y == 0, "none: an orthogonal neighbour ({to})");
}

/// MR1: a ranged skill with a position (range 4) dashes to a position tile and hits from that neighbour; a Maneuver
/// attack without a position carries no move and hits from where it stands; the transition reads a Maneuver attack's
/// `dash_tiles` 2 as flank, range 2 (classic reads no position at all).
#[test]
fn ranged_with_a_position_and_maneuver_without_one() {
    let mut s = session(timeline());
    wolf_board(&mut s);
    place(&mut s.w, LONGBOW, V2::new(2, 3));
    engage(&mut s, LONGBOW);
    assert_eq!(add(&mut s, LONGBOW, "t_dash_shot"), "");
    let e = entry_of(&s, LONGBOW, "t_dash_shot");
    let s_def = &s.w.cat.skills[sid(&s.w, "t_dash_shot")];
    assert_eq!(round_intent::dash_range(&s.w, LONGBOW, s_def), Some(4));
    assert_eq!(e.pos_after.dist(V2::new(6, 4)) <= 2 && e.pos_after != V2::new(6, 4), true, "a neighbour of Wolf: {}", e.pos_after);
    assert!(e.path.len() <= 4, "within the range");
    let hp = s.w.units[WOLF].hp;
    resolve(&mut s);
    assert!(s.w.units[WOLF].hp < hp, "the shot lands from the neighbour");
    let a = s.resolver.scheduled.iter().find(|a| a.unit == LONGBOW).unwrap();
    assert!(matches!(a.command.kind, CmdKind::Attack { min_range: 1, max_range: 1, .. }), "a position hits from the tile next to its target");
    // a Maneuver attack without a position
    let mut s = session(timeline());
    wolf_board(&mut s);
    place(&mut s.w, SPIKER, V2::new(6, 3));
    engage(&mut s, SPIKER);
    assert_eq!(add(&mut s, SPIKER, "t_poke"), "");
    let e = entry_of(&s, SPIKER, "t_poke");
    assert_eq!((e.pos_before, e.pos_after, e.carry), (V2::new(6, 3), V2::new(6, 3), Carry::None));
    // the transition
    let t = &s.w.cat.skills[sid(&s.w, "t_transition")];
    assert_eq!((round_intent::position_of(&s.w, t), round_intent::dash_range(&s.w, SPIKER, t)), (Some(Position::Flank), Some(2)));
    assert_eq!(skill::position_by(t, &Ruleset::classic()), None, "classic reads no position");
}

/// MR14: Disengage hits from the unit's tile (adjacent) and retreats within the skill's range N; AI table row 4 reads
/// "a Maneuver attack in range of an open position tile" (`engage_reach`): false once the flank is protected.
#[test]
fn disengage_retreats_within_the_range_and_ai_row_four_reads_positions() {
    let mut s = session(timeline());
    wolf_board(&mut s);
    place(&mut s.w, SPIKER, V2::new(5, 4));
    round_intent::set_intent(&mut s.w, SPIKER, RoundIntent::Maneuver).unwrap();
    round_intent::set_target(&mut s.w, SPIKER, RoundTarget::Unit(WOLF)).unwrap();
    round_intent::set_maneuver(&mut s.w, SPIKER, ManeuverKind::Disengage).unwrap();
    opener(&mut s, SPIKER, "flank_cut");
    rip::plan_unit(&mut s.w, &mut s.plan, SPIKER).unwrap();
    let retreat = s.plan.entries_for(&s.w, SPIKER).into_iter().find(|e| matches!(e.carry, Carry::Retreat(_))).expect("a retreat");
    assert_eq!(retreat.pos_before, V2::new(5, 4), "the hit from the unit's tile");
    let Carry::Retreat(cell) = retreat.carry else { unreachable!() };
    assert!(cell != V2::new(5, 4) && cell.dist(V2::new(5, 4)) <= 2, "within the range 2: {cell}");
    // AI row 4
    let mut s = session(timeline());
    wolf_board(&mut s);
    let fc = s.w.cat.skills[sid(&s.w, "flank_cut")].clone();
    let none = Cells::new();
    assert!(round_intent::engage_reach(&s.w, SPIKER, &fc, WOLF, V2::new(5, 2), &none));
    assert!(!round_intent::engage_reach(&s.w, SPIKER, &fc, WOLF, V2::new(1, 1), &none), "out of range");
    for (c, cell) in [(GOBLIN, V2::new(6, 3)), (PLAIN, V2::new(6, 5)), (ARCHER, V2::new(5, 3)), (LONGBOW, V2::new(5, 5))] {
        place(&mut s.w, c, cell);
    }
    assert!(!round_intent::engage_reach(&s.w, SPIKER, &fc, WOLF, V2::new(5, 2), &none), "protected");
}

/// MR16: a ranged Aimed Shot whose target stepped 1 tile out of its band follows it 1 tile at 1 AP and lands; so does
/// a heal whose ally stepped out of support reach; root and a hidden target stop the chase; no plan entry is a move.
#[test]
fn every_skill_on_a_unit_chases_one_tile() {
    let shot = |rooted: bool, hidden: bool| {
        let mut s = session(timeline());
        place(&mut s.w, ARCHER, V2::new(3, 4));
        place(&mut s.w, WOLF, V2::new(6, 4));
        round_intent::set_intent(&mut s.w, ARCHER, RoundIntent::Assault).unwrap();
        round_intent::set_target(&mut s.w, ARCHER, RoundTarget::Unit(WOLF)).unwrap();
        let k = sid(&s.w, "tl_aimed_shot");
        assert_eq!(s.plan.add_step(&s.w, ARCHER, PlanStep::engage(k, WOLF), false), "");
        assert!(s.plan.entries_for(&s.w, ARCHER).iter().all(|e| skill::kind(&s.w.cat.skills[e.skill]) != Kind::Move));
        let hp = s.w.units[WOLF].hp;
        assert!(s.confirm_plan());
        drop_enemy_plans(&mut s);
        place(&mut s.w, WOLF, V2::new(7, 4));
        if rooted {
            status::grant_now(&mut s.w.units[ARCHER], "root", 2, 0, "test");
        }
        if hidden {
            status::grant_now(&mut s.w.units[WOLF], "stealth", 2, 0, "test");
            s.w.round_states.entry(WOLF).or_default().moved = true;
        }
        while s.phase != Phase::Replay {
            s.step_next();
        }
        (s.w.pos(ARCHER), s.w.units[ARCHER].ap, s.w.units[WOLF].hp < hp, logged(&s, "[Chase]"))
    };
    let (at, ap, hit, chase) = shot(false, false);
    assert_eq!((at, ap, hit), (V2::new(4, 4), UNIT_AP - 2 - 1, true), "the shot chases 1 tile at 1 AP and lands");
    assert_eq!(chase, ["[Chase] tl_archer closes in on tl_footman: (3, 4) to (4, 4) (1 AP)"]);
    let (at, _, hit, chase) = shot(true, false);
    assert_eq!((at, hit, chase.len()), (V2::new(3, 4), false, 0), "rooted: no chase, out of range");
    let (at, _, _, chase) = shot(false, true);
    assert_eq!((at, chase.len()), (V2::new(3, 4), 0), "hidden: no chase");
    // the heal
    let mut s = session(timeline());
    place(&mut s.w, ARCHER, V2::new(3, 4));
    place(&mut s.w, PLAIN, V2::new(6, 4));
    s.w.units[PLAIN].hp = 200;
    round_intent::set_intent(&mut s.w, ARCHER, RoundIntent::Support).unwrap();
    round_intent::set_target(&mut s.w, ARCHER, RoundTarget::Unit(PLAIN)).unwrap();
    let k = sid(&s.w, "tl_field_dressing");
    assert_eq!(s.plan.add_step(&s.w, ARCHER, PlanStep::ally(k, PLAIN), false), "");
    assert!(s.plan.entries_for(&s.w, ARCHER).iter().all(|e| skill::kind(&s.w.cat.skills[e.skill]) != Kind::Move));
    assert!(s.confirm_plan());
    drop_enemy_plans(&mut s);
    place(&mut s.w, PLAIN, V2::new(7, 4));
    while s.phase != Phase::Replay {
        s.step_next();
    }
    assert_eq!((s.w.pos(ARCHER), s.w.units[ARCHER].ap), (V2::new(4, 4), UNIT_AP - 2 - 1), "the heal chases 1 tile at 1 AP");
    assert!(s.w.units[PLAIN].hp > 200, "and lands");
    let a = s.resolver.scheduled.iter().find(|a| a.unit == ARCHER).unwrap();
    assert_eq!(a.command.moved, Some(MoveKind::Chase));
}
