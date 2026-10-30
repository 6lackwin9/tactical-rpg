//! RULESET1-AI on a scratch catalog built here (no shipped data): the round intent AI of RULESET1_INTENT 9 under the
//! rule `round_intent` (with `enemy_moves`, `ap_cost`, `flank` and `plan_badges`). Expectations come from the rules: each
//! row of the AI1 pick table (flee, Support, Guard and its cover tile, the Skirmisher's Engage, Assault, the idle
//! Guard and Rest), the AUTO1 tactics of AI5 through the session's preset, the intent change of IN6 (a refused pick
//! drops nothing), AI2 (enemies pay no AP), AI4 (tells carry the pick), determinism and classic staying inert;
//! RULESET1-AI2 (no second low-HP Guard round without a hit).

use indexmap::IndexMap;
use rpgkit_battle::ai::{self, IntentRound};
use rpgkit_battle::command::cover_for;
use rpgkit_battle::grid::{V2, TERRAIN_IDS};
use rpgkit_battle::plan::{Mode, Plan};
use rpgkit_battle::round_intent::{self, ManeuverKind, Opener, RoundIntent, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::trigger::Struck;
use rpgkit_battle::world::{AiProfile, Tactic, TargetPref, UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, AP_COST, ENEMY_MOVES, FLANK, PLAN_BADGES, ROUND_INTENT, TIMELINE};
use rpgkit_rules::status;
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
const BASICS: [&str; 4] = ["move", "rest", "strike", "active_guard"];

const BLADE: UnitId = 0;
const BOW: UnitId = 1;
const DANCER: UnitId = 2;
const BRUTE: UnitId = 3;
const HEALER: UnitId = 4;
const SKULK: UnitId = 5;
const ARCHER: UnitId = 6;

/// Engine skills and the test skills (Assault, Guard, Support, Maneuver with carried moves, a DISENGAGE jump).
fn catalog() -> Catalog {
    let atk = |id: &str, intent: &str, blocks: i32, rec: i32, ap: i32, dash: i32| {
        json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": intent, "level": 1, "blocks": blocks, "recovery_blocks": rec, "ap_cost": ap, "power_mult": 1.0, "dash_tiles": dash, "target_mode": "ENEMY"})
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5}),
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "round_intent": "support", "level": 1, "blocks": 2, "ap_cost": 2, "heal_base": 8, "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF"}),
        atk("t_cleave", "assault", 3, 1, 4, 0),
        atk("t_jab", "assault", 1, 0, 1, 0),
        atk("t_shot", "assault", 2, 1, 2, 0),
        atk("t_hamstring", "maneuver", 1, 1, 2, 3),
        atk("t_slip", "maneuver", 2, 1, 2, 2),
        json!({"id": "t_backflip", "name": "Backflip", "kind": "DISENGAGE", "round_intent": "maneuver", "level": 1, "blocks": 1, "ap_cost": 1, "jump_tiles": 2}),
    ];
    let unit = |id: &str, side: &str, range: i32, role: &str, order: serde_json::Value| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": range, "skill_cap": 5, "ai_role": role, "flee_ratio": 0.0, "deck": {"order": order}})
    };
    let mut brute = unit("e_brute", "enemy", 1, "AGGRESSOR", json!({"assault": ["t_cleave", "t_jab"], "support": ["t_mend"]}));
    brute["heal_below"] = json!(0.3);
    brute["defend_below"] = json!(0.5);
    let units = vec![
        unit("p_blade", "party", 1, "AGGRESSOR", json!({"assault": ["t_cleave", "t_jab", "strike"]})),
        unit("p_bow", "party", 3, "SNIPER", json!({"assault": ["t_shot"]})),
        unit("p_dancer", "party", 1, "SKIRMISHER", json!({"assault": ["t_jab"], "maneuver": ["t_slip", "t_backflip"]})),
        brute,
        unit("e_healer", "enemy", 1, "SUPPORT", json!({"support": ["t_mend"], "assault": ["strike"]})),
        unit("e_skulk", "enemy", 1, "SKIRMISHER", json!({"assault": ["t_jab"], "maneuver": ["t_hamstring"]})),
        unit("e_archer", "enemy", 3, "SNIPER", json!({"assault": ["t_shot"]})),
    ];
    Catalog::from_value(&json!({"skills": skills, "units": units}))
}

/// The skills each unit knows (its order lists, Strike and Active Guard): no unit falls back on every catalog skill.
fn known(c: UnitId) -> Vec<&'static str> {
    let own: &[&str] = match c {
        BLADE => &["t_cleave", "t_jab"],
        BOW => &["t_shot"],
        DANCER => &["t_jab", "t_slip", "t_backflip"],
        BRUTE => &["t_cleave", "t_jab", "t_mend"],
        HEALER => &["t_mend"],
        SKULK => &["t_jab", "t_hamstring"],
        _ => &["t_shot"],
    };
    own.iter().copied().chain(["strike", "active_guard"]).collect()
}

fn ruleset(rules: &[&str], intents: Option<&[&str]>) -> Ruleset {
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": BASICS});
    if let Some(i) = intents {
        row["intents"] = json!(i);
    }
    Ruleset::from_row(row.as_object().unwrap())
}

/// The timeline of these tests: the round intent, enemies on recovery without AP, AP costs, flanks and the tells.
fn timeline() -> Ruleset {
    ruleset(&[ROUND_INTENT, ENEMY_MOVES, AP_COST, FLANK, PLAN_BADGES], None)
}

/// Every unit of the catalog in a battle under `rules`, off the field (tests place the ones they use).
fn session(rules: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_blade", "p_bow", "p_dancer"], "enemies": ["e_brute", "e_healer", "e_skulk", "e_archer"],
        "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = rules;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for c in 0..s.w.units.len() {
        s.w.units[c].known_skills = known(c).into_iter().map(String::from).collect();
        s.w.remove(c);
    }
    s
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn hurt(w: &mut World, c: UnitId, share: f64) {
    let u = &mut w.units[c];
    u.hp = (u.max_hp as f64 * share) as i32;
}

fn wall(w: &mut World, cell: V2) {
    let t = TERRAIN_IDS.iter().position(|t| *t == "wall").unwrap();
    w.grid.terrain[(cell.y * rpgkit_battle::grid::GRID_W + cell.x) as usize] = t;
}

/// The AI's round for `c` (its own profile, Balanced) on a fresh plan.
fn pick(w: &mut World, c: UnitId) -> (IntentRound, Plan) {
    let mut plan = Plan::new();
    let p = w.profile_of(c);
    let r = ai::plan_round_intent(w, &mut plan, c, &p, Tactic::Balanced);
    (r, plan)
}

fn intent_target(r: &IntentRound) -> Option<(RoundIntent, RoundTarget)> {
    r.pick.as_ref().map(|p| (p.intent, p.target))
}

fn kind_of(w: &World, sid: usize) -> Kind {
    skill::kind(&w.cat.skills[sid])
}

/// (skill id, reason) per compiled entry of `c`.
fn rows(w: &World, plan: &Plan, c: UnitId) -> Vec<(String, String)> {
    let reasons = rip::reasons(w, plan, c);
    plan.entries_for(w, c).iter().zip(reasons).map(|(e, r)| (w.cat.skills[e.skill].id.clone(), r)).collect()
}

/// Rows 5 and AI2: a healthy Aggressor with nothing else to do Assaults its first ranked target (the nearest), planned
/// from its order with an opener, and pays no AP (enemy side under `enemy_moves`).
#[test]
fn row_5_assault_on_the_first_ranked_target() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, BRUTE, V2::new(6, 4));
    place(w, BLADE, V2::new(4, 4));
    place(w, BOW, V2::new(1, 1));
    let (r, plan) = pick(w, BRUTE);
    assert_eq!(r.row, ai::ROW_ASSAULT);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(BLADE))), "the nearest foe first (Aggressor)");
    assert_eq!(round_intent::intent_of(w, BRUTE), Some(RoundIntent::Assault), "the pick is the unit's intent");
    let entries = plan.entries_for(w, BRUTE);
    assert!(rows(w, &plan, BRUTE).iter().any(|(_, why)| why == rip::OPENER), "{:?}", rows(w, &plan, BRUTE));
    assert!(entries.iter().all(|e| e.total_ap() == 0), "AI2: enemy-side units pay no AP");
    assert!(entries.iter().filter(|e| kind_of(w, e.skill) == Kind::Attack).all(|e| round_intent::skill_intent(&w.cat.skills[e.skill]) == "assault"));
}

/// The end of `c`'s round moves: closer to `foe` than it stands, within the standard move.
fn closes_in(w: &World, plan: &Plan, c: UnitId, foe: UnitId) -> V2 {
    let entries = plan.entries_for(w, c);
    let moves: Vec<_> = entries.iter().filter(|e| kind_of(w, e.skill) == Kind::Move).collect();
    assert!(!moves.is_empty() && kind_of(w, entries[0].skill) == Kind::Move, "the move comes first: {:?}", rows(w, plan, c));
    let end = moves.last().unwrap().pos_after;
    assert!(end.dist(w.pos(foe)) < w.dist(c, foe), "closer: {end}");
    assert!(w.pos(c).dist(end) <= plan.max_move_tiles(w, c), "the standard move");
    end
}

/// Row 5 with no foe in reach this round (the planner offers no Assault opener, PL7; user 2026-10-28): the unit closes
/// in on its first ranked target with the round's standard move and ends by its kind: the front line (here an
/// Aggressor) Guards on the tile it moved to; a reckless unit has no Guard: Assault, the move, Rest; a Support-role unit
/// Supports itself (or an ally) from there.
#[test]
fn row_5_advance_by_kind_when_no_opener_reaches() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, BRUTE, V2::new(11, 4));
    place(w, BLADE, V2::new(0, 4));
    // front line: Guard at the end of the move
    let (r, plan) = pick(w, BRUTE);
    assert_eq!(r.row, ai::ROW_ADVANCE);
    let end = closes_in(w, &plan, BRUTE, BLADE);
    assert_eq!(intent_target(&r), Some((RoundIntent::Guard, RoundTarget::Cell(end))));
    let after: Vec<Kind> = plan.entries_for(w, BRUTE).iter().map(|e| kind_of(w, e.skill)).filter(|k| *k != Kind::Move).collect();
    assert!(after.contains(&Kind::Defend) && after.iter().all(|k| matches!(k, Kind::Defend | Kind::Rest)), "{after:?}");
    // reckless: the move, then Rest (no attack reaches)
    let mut p = w.profile_of(BRUTE);
    p.reckless = true;
    let mut plan = Plan::new();
    let r = ai::plan_round_intent(w, &mut plan, BRUTE, &p, Tactic::Balanced);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_ADVANCE, Some((RoundIntent::Assault, RoundTarget::Unit(BLADE)))));
    closes_in(w, &plan, BRUTE, BLADE);
    assert!(plan.entries_for(w, BRUTE).iter().all(|e| matches!(kind_of(w, e.skill), Kind::Move | Kind::Rest)), "{:?}", rows(w, &plan, BRUTE));
    assert_eq!(rows(w, &plan, BRUTE)[0].1, rip::APPROACH);
    // support: a Support skill on itself after the move
    w.remove(BRUTE);
    place(w, HEALER, V2::new(11, 4));
    let (r, plan) = pick(w, HEALER);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_ADVANCE, Some((RoundIntent::Support, RoundTarget::Myself))));
    closes_in(w, &plan, HEALER, BLADE);
    assert!(plan.entries_for(w, HEALER).iter().any(|e| kind_of(w, e.skill) == Kind::Support), "{:?}", rows(w, &plan, HEALER));
}

/// Row 5b for a ranged unit (user 2026-10-28): after the move it uses a self skill (a Support skill on itself) when it
/// has one usable this round, and Guards only without one.
#[test]
fn row_5_advance_ranged_uses_a_self_skill_else_guards() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, ARCHER, V2::new(11, 4));
    place(w, BLADE, V2::new(0, 4));
    // without a self skill: Guard on the tile it moved to
    let (r, plan) = pick(w, ARCHER);
    assert_eq!(r.row, ai::ROW_ADVANCE);
    let end = closes_in(w, &plan, ARCHER, BLADE);
    assert_eq!(intent_target(&r), Some((RoundIntent::Guard, RoundTarget::Cell(end))));
    // with one (Mend, a Support skill that reaches its user): Support on itself after the move
    w.units[ARCHER].known_skills.push("t_mend".into());
    let (r, plan) = pick(w, ARCHER);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_ADVANCE, Some((RoundIntent::Support, RoundTarget::Myself))));
    closes_in(w, &plan, ARCHER, BLADE);
    assert!(plan.entries_for(w, ARCHER).iter().any(|e| w.cat.skills[e.skill].id == "t_mend"), "{:?}", rows(w, &plan, ARCHER));
}

/// Row 2: a Support-role unit picks Support on the hurt friend with the lowest HP share; with nobody hurt it does not.
/// A unit of another role picks Support only for a friend at or below its `heal_below`.
#[test]
fn row_2_support_on_the_lowest_hp_share() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, HEALER, V2::new(8, 4));
    place(w, BRUTE, V2::new(8, 2));
    place(w, ARCHER, V2::new(9, 6));
    place(w, BLADE, V2::new(3, 4));
    let (r, _) = pick(w, HEALER);
    assert_ne!(r.pick.as_ref().map(|p| p.intent), Some(RoundIntent::Support), "nobody hurt: no Support round ({r:?})");
    hurt(w, BRUTE, 0.9);
    hurt(w, ARCHER, 0.6);
    let (r, plan) = pick(w, HEALER);
    assert_eq!(r.row, ai::ROW_SUPPORT);
    assert_eq!(intent_target(&r), Some((RoundIntent::Support, RoundTarget::Unit(ARCHER))), "the lowest HP share");
    assert!(plan.entries_for(w, HEALER).iter().any(|e| w.cat.skills[e.skill].id == "t_mend"));
    // the brute (Aggressor) heals only below its heal_below (0.3 in the catalog)
    let (r, _) = pick(w, BRUTE);
    assert_ne!(r.row, ai::ROW_SUPPORT, "0.6 is above the brute's heal_below ({r:?})");
    hurt(w, ARCHER, 0.25);
    let (r, _) = pick(w, BRUTE);
    assert_eq!(intent_target(&r), Some((RoundIntent::Support, RoundTarget::Unit(ARCHER))));
}

/// Row 3: at or below `defend_below` (not reckless) a unit Guards itself while no ranged foe threatens it.
#[test]
fn row_3_guard_self_without_a_shooter() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, ARCHER, V2::new(6, 4));
    place(w, BLADE, V2::new(4, 5));
    hurt(w, ARCHER, 0.3);
    let (r, plan) = pick(w, ARCHER);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_GUARD, Some((RoundIntent::Guard, RoundTarget::Myself))), "no shooter: Guard self");
    assert!(plan.entries_for(w, ARCHER).iter().all(|e| matches!(kind_of(w, e.skill), Kind::Defend | Kind::Rest)));
}

/// The tiles `c` may move to this round on open ground (its own and the free walkable tiles within `blocks` move
/// blocks), each with the foes within `c`'s reach band from it, its threats and its cover against `shooter`.
fn tiles(w: &World, c: UnitId, blocks: i32, shooter: UnitId) -> Vec<(V2, usize, usize, f64)> {
    let here = w.pos(c);
    let band = skill::reach_by(&w.cat.skills[w.book().lowest(Kind::Attack)], &w.units[c], &w.ruleset);
    let foes: Vec<V2> = w.foes_of(c).iter().filter(|&&f| w.has(f)).map(|&f| w.pos(f)).collect();
    let mut out = Vec::new();
    for y in 0..rpgkit_battle::grid::GRID_H {
        for x in 0..rpgkit_battle::grid::GRID_W {
            let cell = V2::new(x, y);
            if cell != here && (cell.dist(here) > w.units[c].move_per_block() * blocks || !w.grid.is_walkable(cell) || w.is_occupied(cell)) {
                continue;
            }
            let hits = foes.iter().filter(|f| f.dist(cell) >= band.0 && f.dist(cell) <= band.1).count();
            out.push((cell, hits, round_intent::threat(w, c, cell), cover_for(&w.grid, w.pos(shooter), cell, true)));
        }
    }
    out
}

fn cover_walls(w: &mut World, cells: &[V2]) {
    let t = TERRAIN_IDS.iter().position(|t| *t == "wall").unwrap();
    w.grid.table.rows[t].cover_guard = 0.5;
    for &c in cells {
        wall(w, c);
    }
}

/// The tile `c`'s round moves to first (its first entry is a move).
fn moved_to(w: &World, plan: &Plan, c: UnitId) -> V2 {
    let entries = plan.entries_for(w, c);
    let moves: Vec<_> = entries.iter().take_while(|e| kind_of(w, e.skill) == Kind::Move).collect();
    moves.last().unwrap_or_else(|| panic!("a move first: {:?}", rows(w, plan, c))).pos_after
}

/// The tile ranking, tier 2 (user 2026-10-28): a ranged unit whose selected target (its first ranked, here the weakest,
/// far off) is out of reach this round repositions to a tile from which it hits as many hostiles as possible, and
/// Assaults one of them from there.
#[test]
fn tier_2_ranged_hits_the_most_hostiles() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, ARCHER, V2::new(6, 4));
    place(w, BOW, V2::new(0, 8));
    place(w, BLADE, V2::new(10, 2));
    place(w, DANCER, V2::new(10, 6));
    hurt(w, BOW, 0.1);
    let (r, plan) = pick(w, ARCHER);
    assert_eq!(r.row, ai::ROW_ASSAULT);
    let Some((RoundIntent::Assault, RoundTarget::Unit(t))) = intent_target(&r) else { panic!("{r:?}") };
    assert_ne!(t, BOW, "the selected target is out of reach");
    let cell = moved_to(w, &plan, ARCHER);
    let all = tiles(w, ARCHER, 2, BOW);
    let most = all.iter().map(|t| t.1).max().unwrap();
    assert!(most >= 2, "the fixture: a tile hits both");
    assert_eq!(all.iter().find(|t| t.0 == cell).unwrap().1, most, "{cell}: the most hostiles in reach");
    assert!(plan.entries_for(w, ARCHER).iter().any(|e| kind_of(w, e.skill) == Kind::Attack), "{:?}", rows(w, &plan, ARCHER));
}

/// Tier 2 for a melee unit: close quarters with as many hostiles as possible (the tile adjacent to the most).
#[test]
fn tier_2_melee_closes_with_the_most_hostiles() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, BRUTE, V2::new(6, 4));
    place(w, BOW, V2::new(0, 8));
    place(w, BLADE, V2::new(9, 3));
    place(w, DANCER, V2::new(9, 5));
    hurt(w, BOW, 0.1);
    let mut p = w.profile_of(BRUTE);
    p.target_pref = TargetPref::Weakest;
    let mut plan = Plan::new();
    let r = ai::plan_round_intent(w, &mut plan, BRUTE, &p, Tactic::Balanced);
    assert_eq!(r.row, ai::ROW_ASSAULT);
    assert_ne!(intent_target(&r).map(|x| x.1), Some(RoundTarget::Unit(BOW)), "the selected target is out of reach");
    let cell = moved_to(w, &plan, BRUTE);
    let all = tiles(w, BRUTE, 2, BOW);
    let most = all.iter().map(|t| t.1).max().unwrap();
    assert!(most >= 2, "the fixture: a tile touches both");
    assert_eq!(all.iter().find(|t| t.0 == cell).unwrap().1, most, "{cell}: adjacent to the most hostiles");
}

/// Tier 3: at low HP (row 3) a unit threatened by a shooter looks to its safety: the tile within one move block with
/// the fewest threats, then the most cover against the shooter.
#[test]
fn tier_3_low_hp_takes_safety() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, ARCHER, V2::new(6, 4));
    place(w, BOW, V2::new(2, 4));
    hurt(w, ARCHER, 0.3);
    cover_walls(w, &[V2::new(4, 4), V2::new(6, 3)]);
    let (r, plan) = pick(w, ARCHER);
    assert_eq!(r.row, ai::ROW_GUARD);
    let Some((RoundIntent::Guard, RoundTarget::Cell(cell))) = intent_target(&r) else { panic!("a safe tile: {r:?}") };
    assert_eq!(rows(w, &plan, ARCHER)[0].1, rip::COVER_TILE);
    let all = tiles(w, ARCHER, 1, BOW);
    let fewest = all.iter().map(|t| t.2).min().unwrap();
    let best = all.iter().filter(|t| t.2 == fewest).map(|t| t.3).fold(0.0, f64::max);
    let (_, _, threat, cover) = *all.iter().find(|t| t.0 == cell).unwrap();
    assert_eq!(threat, fewest, "{cell}: the fewest threats");
    assert!(cover == best && cover > 0.0, "{cell}: the most cover among them");
}

/// MM1 with the tactics (user 2026-10-28): a member set to Manual keeps its intent, target and steps when a tactic is
/// applied to the party or to it; the others are drafted.
#[test]
fn a_tactic_skips_manual_members() {
    let mut s = session(timeline());
    {
        let w = &mut s.w;
        place(w, BLADE, V2::new(4, 4));
        place(w, BOW, V2::new(2, 2));
        place(w, BRUTE, V2::new(6, 4));
    }
    s.set_round_intent(BLADE, "guard").unwrap();
    s.set_round_target(BLADE, RoundTarget::Myself).unwrap();
    rip::set_manual(&mut s.w, &mut s.plan, BLADE, true).unwrap();
    let guard = s.w.book().find("active_guard").unwrap();
    assert_eq!(s.plan.add_step(&s.w, BLADE, rpgkit_battle::plan::PlanStep::self_skill(guard), false), "");
    let before = (round_intent::intent_of(&s.w, BLADE), s.w.round_states[&BLADE].target, rows(&s.w, &s.plan, BLADE));
    s.plan_apply_preset_all("aggressive");
    assert_eq!((round_intent::intent_of(&s.w, BLADE), s.w.round_states[&BLADE].target, rows(&s.w, &s.plan, BLADE)), before, "the manual member is left alone");
    assert_eq!(round_intent::intent_of(&s.w, BOW), Some(RoundIntent::Assault), "the others are drafted");
    s.plan_apply_preset(BLADE, "aggressive");
    assert_eq!((round_intent::intent_of(&s.w, BLADE), s.w.round_states[&BLADE].target, rows(&s.w, &s.plan, BLADE)), before, "also one by one");
}

/// Row 4: a Skirmisher whose Maneuver attack reaches a flank tile on a ranked target engaged with another unit picks
/// Maneuver Engage on it; with the target engaged with nobody it Assaults.
#[test]
fn row_4_skirmisher_engages_a_flank() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, BLADE, V2::new(5, 4));
    place(w, BRUTE, V2::new(6, 4));
    place(w, SKULK, V2::new(6, 6));
    let (r, _) = pick(w, SKULK);
    assert_eq!(r.row, ai::ROW_ASSAULT, "the blade is engaged with nobody yet");
    w.units[BLADE].active_target = Some(BRUTE);
    round_intent::update_facing(w, BLADE);
    assert_eq!(round_intent::engaged_with(w, BLADE), Some(BRUTE));
    let (r, plan) = pick(w, SKULK);
    assert_eq!(r.row, ai::ROW_ENGAGE);
    assert_eq!(intent_target(&r), Some((RoundIntent::Maneuver, RoundTarget::Unit(BLADE))));
    assert_eq!(r.pick.as_ref().unwrap().kind, Some(ManeuverKind::Engage));
    let hit = plan.entries_for(w, SKULK).into_iter().find(|e| kind_of(w, e.skill) == Kind::Attack).expect("an attack");
    assert!(!round_intent::in_front(w, BLADE, hit.pos_after), "the hit lands from outside the blade's front ({})", hit.pos_after);
}

/// Row 1: at or below `flee_ratio` a unit Withdraws (a neutral move) and its intent stays.
#[test]
fn row_1_flee_withdraws_and_keeps_the_intent() {
    let mut s = session(timeline());
    let w = &mut s.w;
    w.grid.exits = vec![V2::new(11, 4)];
    place(w, BRUTE, V2::new(9, 4));
    place(w, BLADE, V2::new(3, 4));
    round_intent::set_intent(w, BRUTE, RoundIntent::Guard).unwrap();
    hurt(w, BRUTE, 0.2);
    let mut p = w.profile_of(BRUTE);
    p.flee_ratio = 0.25;
    let mut plan = Plan::new();
    let r = ai::plan_round_intent(w, &mut plan, BRUTE, &p, Tactic::Balanced);
    assert_eq!((r.row, r.pick.clone()), (ai::ROW_FLEE, None));
    assert!(plan.steps_of(BRUTE).iter().all(|st| st.mode == Mode::Withdraw), "{:?}", plan.steps_of(BRUTE).iter().map(|s| s.mode).collect::<Vec<_>>());
    assert!(plan.compile(w, BRUTE).withdrawn);
    assert_eq!(round_intent::intent_of(w, BRUTE), Some(RoundIntent::Guard), "intent unchanged");
    p.reckless = true;
    let r = ai::plan_round_intent(w, &mut plan, BRUTE, &p, Tactic::Balanced);
    assert_ne!(r.row, ai::ROW_FLEE, "a reckless unit does not flee");
}

/// Row 6: nothing reaches any foe: Guard self; with no Guard skill in the ruleset's intents: a Rest round.
#[test]
fn row_6_idle_guard_else_rest() {
    for (intents, row) in [(None, ai::ROW_IDLE_GUARD), (Some(&["assault", "support", "maneuver"][..]), ai::ROW_REST)] {
        let mut s = session(ruleset(&[ROUND_INTENT, ENEMY_MOVES, AP_COST], intents));
        let w = &mut s.w;
        for y in 0..9 {
            wall(w, V2::new(5, y));
        }
        place(w, BRUTE, V2::new(8, 4));
        place(w, BLADE, V2::new(2, 4));
        let (r, plan) = pick(w, BRUTE);
        assert_eq!(r.row, row, "intents {intents:?}");
        let entries = plan.entries_for(w, BRUTE);
        assert!(!entries.is_empty());
        if row == ai::ROW_REST {
            assert!(entries.iter().all(|e| kind_of(w, e.skill) == Kind::Rest));
            assert_eq!(entries.len() as i32, BLOCKS, "every block a Rest");
        } else {
            assert_eq!(intent_target(&r), Some((RoundIntent::Guard, RoundTarget::Myself)));
        }
    }
}

/// IN6 with the AI: a refused pick (Support on a friend out of reach) drops no intent-bound buff; a real intent change
/// does and returns its line.
#[test]
fn a_refused_pick_drops_nothing_a_change_drops_riposte() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, BRUTE, V2::new(1, 4));
    place(w, SKULK, V2::new(11, 4));
    place(w, BLADE, V2::new(3, 4));
    round_intent::set_intent(w, BRUTE, RoundIntent::Guard).unwrap();
    status::grant_now(&mut w.units[BRUTE], "riposte", 2, 0, "test");
    round_intent::bind(w, BRUTE, BRUTE, &["riposte".to_string()], false);
    hurt(w, SKULK, 0.2);
    hurt(w, BRUTE, 0.45);
    let (r, _) = pick(w, BRUTE);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_GUARD, Some((RoundIntent::Guard, RoundTarget::Myself))), "Support on the far skulk was refused");
    assert!(r.lines.is_empty() && w.units[BRUTE].has_status("riposte"), "{:?}", r.lines);
    w.units[BRUTE].hp = w.units[BRUTE].max_hp;
    let (r, _) = pick(w, BRUTE);
    assert_eq!(r.row, ai::ROW_ASSAULT);
    assert!(!w.units[BRUTE].has_status("riposte"));
    assert!(r.lines.len() == 1 && r.lines[0].contains("leaves Guard"), "{:?}", r.lines);
}

/// A pick the trial offers but the real set refuses falls to the next row: the brute's Assault reaches the blade only
/// with its Guard-bound haste (one more tile a move block), which the change to Assault drops (IN6); the table goes on
/// to the advance, and the intent change line is kept.
#[test]
fn a_pick_that_fails_for_real_falls_to_the_next_row() {
    let setup = |bound: bool| {
        let mut s = session(timeline());
        let w = &mut s.w;
        place(w, BRUTE, V2::new(11, 4));
        place(w, BLADE, V2::new(5, 4));
        round_intent::set_intent(w, BRUTE, RoundIntent::Guard).unwrap();
        status::grant_now(&mut w.units[BRUTE], "haste", 2, 0, "test");
        round_intent::bind(w, BRUTE, BRUTE, &["haste".to_string()], bound);
        s
    };
    // the control: unbound haste stays, so the trial's Assault (which keeps every buff) is offered and lands
    let mut c = setup(false);
    assert_eq!(pick(&mut c.w, BRUTE).0.row, ai::ROW_ASSAULT);
    let mut s = setup(true);
    let w = &mut s.w;
    let hasted = w.units[BRUTE].move_per_block();
    let band = skill::reach_by(&w.cat.skills[w.book().find("t_jab").unwrap()], &w.units[BRUTE], &w.ruleset).1;
    let need = w.dist(BRUTE, BLADE) - band;
    assert!(need <= hasted * 2 && need > (hasted - 1) * 2, "the fixture: in reach only while hasted (need {need}, {hasted} a block)");
    let (r, plan) = pick(w, BRUTE);
    assert_eq!(r.row, ai::ROW_ADVANCE, "{r:?}");
    let end = closes_in(w, &plan, BRUTE, BLADE);
    assert_eq!(intent_target(&r), Some((RoundIntent::Guard, RoundTarget::Cell(end))), "the front line Guards after the move");
    assert!(!w.units[BRUTE].has_status("haste"));
    assert!(r.lines.iter().any(|l| l.contains("leaves Guard")), "{:?}", r.lines);
    assert!(!plan.entries_for(w, BRUTE).is_empty(), "the advance planned the round");
}

/// An Economy pick whose round has no opener step (here an advance's round: a move, then follow-ups) is not an Economy
/// round: the commit fails and the table picks from its first row; the failed commit leaves the previous lock (its
/// `set_target` had locked on the pick's target).
#[test]
fn economy_without_an_opener_falls_back_to_the_table() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, BRUTE, V2::new(6, 4));
    place(w, BLADE, V2::new(4, 4));
    place(w, DANCER, V2::new(9, 4));
    hurt(w, BRUTE, 0.4);
    w.units[BRUTE].active_target = Some(DANCER);
    let pick = ai::IntentPick {
        intent: RoundIntent::Assault,
        target: RoundTarget::Unit(BLADE),
        kind: None,
        opener: None,
        opener_only: true,
        advance_to: Some(V2::new(5, 4)),
        must_act: false,
    };
    let mut plan = Plan::new();
    let p = w.profile_of(BRUTE);
    let r = ai::plan_given(w, &mut plan, BRUTE, &p, pick);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_GUARD, Some((RoundIntent::Guard, RoundTarget::Myself))), "the table's row 3 (hurt, no shooter)");
    assert_eq!(w.units[BRUTE].active_target, Some(DANCER), "the previous lock");
    assert!(!rows(w, &plan, BRUTE).iter().any(|(_, why)| why == rip::APPROACH), "nothing of the failed pick is left");
}

/// AI5 through the session's preset (the host's `preset` command): each tactic's intent and target.
#[test]
fn tactics_map_to_intents() {
    let mut s = session(timeline());
    {
        let w = &mut s.w;
        place(w, BLADE, V2::new(4, 4));
        place(w, BOW, V2::new(7, 7));
        place(w, DANCER, V2::new(7, 2));
        place(w, BRUTE, V2::new(6, 4));
        place(w, SKULK, V2::new(7, 6));
        place(w, ARCHER, V2::new(8, 2));
        hurt(w, SKULK, 0.5);
    }
    let pick_of = |s: &Session, c: UnitId| (round_intent::intent_of(&s.w, c), s.w.round_states.get(&c).and_then(|r| r.target));
    // All-out Attack: Assault on the first ranked target (nearest)
    s.plan_apply_preset(BLADE, "aggressive");
    assert_eq!(pick_of(&s, BLADE), (Some(RoundIntent::Assault), Some(RoundTarget::Unit(BRUTE))));
    // Focus Weakest: the lowest-HP foe reachable this round
    s.plan_apply_preset(BLADE, "focus_weakest");
    assert_eq!(pick_of(&s, BLADE), (Some(RoundIntent::Assault), Some(RoundTarget::Unit(SKULK))));
    // Guard & Counter: Guard self
    s.plan_apply_preset(BLADE, "defensive");
    assert_eq!(pick_of(&s, BLADE), (Some(RoundIntent::Guard), Some(RoundTarget::Myself)));
    // Kite at Range: the ranged member Assaults its weakest-first target, kiting out of its melee band first (AS2)
    s.plan_apply_preset(BOW, "ranged_kite");
    assert_eq!(pick_of(&s, BOW), (Some(RoundIntent::Assault), Some(RoundTarget::Unit(SKULK))));
    assert!(s.w.dist(BOW, SKULK) <= round_intent::melee_reach(&s.w, SKULK));
    let bow_rows = rows(&s.w, &s.plan, BOW);
    assert_eq!(bow_rows[0].1, rip::KITE_IN_MELEE, "{bow_rows:?}");
    // ... and the melee member Disengages from the foe in its band
    s.plan_apply_preset(DANCER, "ranged_kite");
    assert_eq!(pick_of(&s, DANCER).0, Some(RoundIntent::Maneuver));
    assert_eq!(round_intent::maneuver_of(&s.w, DANCER), ManeuverKind::Disengage);
    // Economy: the cheapest Assault opener, then Rest
    s.plan_apply_preset(BLADE, "economy");
    assert_eq!(pick_of(&s, BLADE).0, Some(RoundIntent::Assault));
    let w = &s.w;
    let cheapest = rip::order(w, BLADE, RoundIntent::Assault).into_iter().map(|sid| skill::ap_cost_by(&w.cat.skills[sid], Some(&w.units[BLADE]), &w.ruleset)).min().unwrap();
    let entries = s.plan.entries_for(w, BLADE);
    let attacks: Vec<_> = entries.iter().filter(|e| kind_of(w, e.skill) == Kind::Attack).collect();
    assert_eq!(attacks.len(), 1, "no follow-ups: {:?}", rows(w, &s.plan, BLADE));
    assert_eq!(attacks[0].total_ap(), cheapest);
    let after = entries.iter().skip_while(|e| kind_of(w, e.skill) != Kind::Attack).skip(1);
    assert!(after.into_iter().all(|e| kind_of(w, e.skill) == Kind::Rest));
    // Balanced is the AI table: a member at or below defend_below Guards
    hurt(&mut s.w, BLADE, 0.3);
    s.plan_apply_preset_all("balanced");
    assert_eq!(pick_of(&s, BLADE).0, Some(RoundIntent::Guard));
    assert!(s.plan_is_valid(), "the drafted party plan confirms");
}

/// An explicit opener is honoured: Economy's pick names the opener the planner placed.
#[test]
fn economy_names_its_opener() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, BLADE, V2::new(4, 4));
    place(w, BRUTE, V2::new(5, 4));
    let mut plan = Plan::new();
    let p = w.profile_of(BLADE).with_preset("economy");
    let r = ai::plan_round_intent(w, &mut plan, BLADE, &p, Tactic::Economy);
    let Some(Opener::Skill(sid)) = r.pick.as_ref().and_then(|p| p.opener.clone()) else { panic!("an opener: {r:?}") };
    let first_attack = plan.entries_for(w, BLADE).into_iter().find(|e| kind_of(w, e.skill) == Kind::Attack).unwrap();
    assert_eq!(first_attack.skill, sid);
}

/// AI1, AI4 on a whole enemy side through the session: the enemies vary their intents by the table (Assault, Support,
/// Maneuver, Guard), each tell carries its unit's intent and target, and the enemy plans are deterministic.
#[test]
fn the_enemy_side_varies_its_intents_and_tells_show_them() {
    let setup = || {
        let mut s = session(timeline());
        let w = &mut s.w;
        place(w, BLADE, V2::new(5, 4));
        place(w, BOW, V2::new(1, 2));
        place(w, BRUTE, V2::new(6, 4));
        place(w, HEALER, V2::new(9, 1));
        place(w, SKULK, V2::new(6, 6));
        place(w, ARCHER, V2::new(9, 6));
        hurt(w, ARCHER, 0.35);
        w.units[BLADE].active_target = Some(BRUTE);
        s.replan_enemies();
        s
    };
    let s = setup();
    let intents: Vec<Option<RoundIntent>> = [BRUTE, HEALER, SKULK, ARCHER].iter().map(|&e| round_intent::intent_of(&s.w, e)).collect();
    assert_eq!(intents, vec![Some(RoundIntent::Assault), Some(RoundIntent::Support), Some(RoundIntent::Maneuver), Some(RoundIntent::Guard)]);
    let tells = s.enemy_intents();
    for e in [BRUTE, HEALER, SKULK, ARCHER] {
        let t = tells.iter().find(|r| r.unit == e).expect("a tell");
        assert_eq!(t.round_intent, round_intent::intent_of(&s.w, e), "AI4: {}", s.w.units[e].name);
        assert_eq!(t.round_target, s.w.round_states[&e].target, "AI4: {}", s.w.units[e].name);
    }
    let again = setup();
    let titles = |s: &Session| s.resolver.scheduled.iter().map(|a| (a.unit, a.command.name.clone(), a.start, a.end)).collect::<Vec<_>>();
    assert_eq!(titles(&s), titles(&again), "deterministic");
}

/// Classic is untouched: the round intent AI is inert without the rule (no pick, no steps) and the session's enemies
/// keep their classic draft (no round state).
#[test]
fn classic_is_inert() {
    let mut s = session(Ruleset::classic());
    let w = &mut s.w;
    place(w, BRUTE, V2::new(6, 4));
    place(w, BLADE, V2::new(4, 4));
    let (r, plan) = pick(w, BRUTE);
    assert_eq!(r, IntentRound::default());
    assert!(plan.steps_of(BRUTE).is_empty());
    assert!(w.round_states.is_empty());
    let mut plan = Plan::new();
    let p: AiProfile = w.profile_of(BLADE).with_preset("aggressive");
    assert_eq!(ai::plan_round_intent(w, &mut plan, BLADE, &p, Tactic::parse("aggressive")), IntentRound::default(), "tactics too");
    assert_eq!(Tactic::parse("withdraw"), Tactic::Balanced, "other presets draft by the table");
}

// ---- RULESET1-AI2: no second low-HP Guard round without a hit ---------------------------------------------------------

/// RULESET1_BALANCE1 AI2-1, AI2-2: row 3 is skipped for a unit that was in the Guard intent last round and lost no HP
/// to an attack hit during it (the table goes on down: here Assault); a round in Guard that took a hit, or a round not
/// in Guard, keeps row 3.
#[test]
fn row_3_no_second_guard_round_without_a_hit() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, ARCHER, V2::new(6, 4));
    place(w, BLADE, V2::new(4, 5));
    hurt(w, ARCHER, 0.3);
    let guard = (ai::ROW_GUARD, Some((RoundIntent::Guard, RoundTarget::Myself)));
    let (r, _) = pick(w, ARCHER);
    assert_eq!((r.row, intent_target(&r)), guard, "the fixture: row 3 fires");
    // the round in Guard ends without a hit: no Guard this round
    w.roll_hits();
    assert_eq!(w.hits.guarded_last_round, vec![ARCHER]);
    let (r, _) = pick(w, ARCHER);
    assert_ne!(r.row, ai::ROW_GUARD);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(BLADE))), "down the table: Assault");
    // that round was not in Guard: row 3 may fire again
    w.roll_hits();
    assert!(w.hits.guarded_last_round.is_empty());
    let (r, _) = pick(w, ARCHER);
    assert_eq!((r.row, intent_target(&r)), guard, "the round after: Guard again");
    // a round in Guard that took a hit: Guard allowed
    w.note_hits(&[Struck { target: ARCHER, damage: 5, ranged: false }]);
    w.roll_hits();
    assert_eq!((w.hits.guarded_last_round.clone(), w.hits.last_round.clone()), (vec![ARCHER], vec![ARCHER]));
    let (r, _) = pick(w, ARCHER);
    assert_eq!((r.row, intent_target(&r)), guard, "hit while guarding: Guard again");
    // a hit that took no HP is no hit (the hit memory counts HP lost)
    w.note_hits(&[Struck { target: ARCHER, damage: 0, ranged: false }]);
    w.roll_hits();
    let (r, _) = pick(w, ARCHER);
    assert_ne!(r.row, ai::ROW_GUARD, "a blocked hit bought nothing");
}

/// The acceptance case through a session: the Brute at 40 % HP (under `defend_below`, over `heal_below`) beside an idle
/// foe Guards round 1, takes no hit, and does not Guard round 2.
#[test]
fn a_session_guard_round_without_a_hit_is_not_repeated() {
    let mut s = session(timeline());
    place(&mut s.w, BRUTE, V2::new(5, 4));
    place(&mut s.w, BLADE, V2::new(4, 4));
    hurt(&mut s.w, BRUTE, 0.4);
    s.replan_enemies();
    assert_eq!(round_intent::intent_of(&s.w, BRUTE), Some(RoundIntent::Guard), "the fixture: round 1 Guards");
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
    assert!(s.can_advance_round());
    s.advance_round();
    assert!(s.w.hits.guarded_unhit(BRUTE), "{:?}", s.w.hits);
    s.replan_enemies();
    assert_ne!(round_intent::intent_of(&s.w, BRUTE), Some(RoundIntent::Guard), "round 2: no second Guard round");
}

/// Classic is untouched: no Guard memory is kept and the classic draft of a hurt unit is the same with or without one.
#[test]
fn classic_keeps_no_guard_memory() {
    let mut s = session(Ruleset::classic());
    let w = &mut s.w;
    place(w, BRUTE, V2::new(6, 4));
    place(w, BLADE, V2::new(4, 4));
    hurt(w, BRUTE, 0.4);
    w.note_hits(&[Struck { target: BRUTE, damage: 5, ranged: false }]);
    w.roll_hits();
    assert_eq!(w.hits, Default::default(), "classic: nothing recorded");
    let draft = |w: &World| format!("{:?}", ai::build_steps(w, &mut Plan::new(), BRUTE, &w.profile_of(BRUTE)));
    let plain = draft(w);
    w.hits.guarded_last_round = vec![BRUTE];
    assert_eq!(draft(w), plain);
}
