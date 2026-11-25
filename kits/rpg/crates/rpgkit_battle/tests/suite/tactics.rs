//! HOOKS1-AI on a scratch catalog built here (no shipped data): AI target preferences, unit tactics and hazard-aware
//! tiles under the rule `ai_tactics` (HOOKS1 11). Expectations come from the rules: SOFTEST ranks the lowest armour
//! rating first and the foes out of reach this round last, TOUGHEST the melee foe with the most max HP first, a
//! reachable taunter still first (AT1, in the shared `rank_targets` of both AIs); `brace_after_hit` Guards when the unit
//! or a friend within 2 tiles lost HP to an attack hit last round (a row before AI table row 4), on its `hold_line`
//! tile when it has one; `hold_line` ranks PL4 tiles next to a friend first, then fewer free neighbours; `flush_cover`
//! scores a foe hit in cover once more (the classic area aim and the round intent AI's area opener); PL4's threat counts
//! a visible harmful ground hazard (AT2, AT3). With the rule off SOFTEST and TOUGHEST read as ROLE and nothing else
//! changes: the same plans as a unit without the fields (4.1, 4.2).

use indexmap::IndexMap;
use rpgkit_battle::ai::{self, IntentRound};
use rpgkit_battle::grid::{V2, TERRAIN_IDS};
use rpgkit_battle::ground::Placed;
use rpgkit_battle::plan::{Mode, Plan, Step};
use rpgkit_battle::round_intent::{self, Aim, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::trigger::Struck;
use rpgkit_battle::world::{AiProfile, Tactic, TargetPref, UnitId, UnitTactic, World};
use rpgkit_data::Catalog;
use rpgkit_rules::damage;
use rpgkit_rules::ruleset::{Ruleset, AI_TACTICS, AP_COST, ENEMY_MOVES, GROUND_HAZARDS, ROUND_INTENT, TIMELINE};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const BASICS: [&str; 4] = ["move", "rest", "strike", "active_guard"];

const PLATE: UnitId = 0;
const CLOTH: UnitId = 1;
const BOW: UnitId = 2;
const FLAG: UnitId = 3;
const BAILIFF: UnitId = 4;
const FOOTMAN: UnitId = 5;
const MATE: UnitId = 6;
const PURIFIER: UnitId = 7;

/// Engine skills, a melee jab, a ranged shot and a PLUS area attack (reach 2-4); the slice's knobs on the enemies.
fn catalog() -> Catalog {
    let atk = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 1, "recovery_blocks": 0, "ap_cost": 1, "power_mult": 1.0, "target_mode": "ENEMY"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
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
        atk("t_jab", json!({})),
        atk("t_shot", json!({"blocks": 2})),
        atk("t_blast", json!({"blocks": 2, "area": "PLUS", "reach_min": 2, "reach_max": 4})),
    ];
    let unit = |id: &str, side: &str, hp: i32, armor: i32, range: i32, order: Value, extra: Value| {
        let mut row = json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": hp, "armor": armor, "speed": 12, "attack_range": range,
            "skill_cap": 5, "ai_role": "AGGRESSOR", "flee_ratio": 0.0, "deck": {"order": order}});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let jab = || json!({"assault": ["t_jab"]});
    let units = vec![
        unit("p_plate", "party", 230, 25, 1, jab(), json!({})),
        unit("p_cloth", "party", 230, 6, 1, jab(), json!({})),
        unit("p_bow", "party", 250, 6, 3, json!({"assault": ["t_shot"]}), json!({})),
        unit("e_flag", "enemy", 300, 0, 1, jab(), json!({"target_pref": "SOFTEST"})),
        unit("e_bailiff", "enemy", 300, 0, 1, jab(), json!({"target_pref": "TOUGHEST"})),
        unit("e_footman", "enemy", 300, 0, 1, jab(), json!({"tactics": ["brace_after_hit", "hold_line"]})),
        unit("e_mate", "enemy", 300, 0, 1, jab(), json!({})),
        unit("e_purifier", "enemy", 300, 0, 1, json!({"assault": ["t_blast", "t_jab"]}), json!({"target_pref": "WEAKEST", "tactics": ["flush_cover"]})),
    ];
    Catalog::from_value(&json!({
        "skills": skills,
        "units": units,
        "hazards": [
            {"id": "g_fire", "name": "Phosphorus Fire", "on": "round_end", "status": "burn", "potency": 80, "magnitude": 5, "status_rounds": 2, "damage": 8, "duration": 2},
            {"id": "g_trip", "name": "Tripwire", "on": "enter", "damage": 15, "trap": true, "hidden": true, "duration": 3},
            {"id": "g_smoke", "name": "Soot Smoke", "on": "both", "cover_guard": 0.3, "duration": 2}
        ]
    }))
}

/// The skills each unit knows (its order list, Strike and Active Guard).
fn known(c: UnitId) -> Vec<&'static str> {
    let own: &[&str] = match c {
        BOW => &["t_shot"],
        PURIFIER => &["t_blast", "t_jab"],
        _ => &["t_jab"],
    };
    own.iter().copied().chain(["strike", "active_guard"]).collect()
}

fn ruleset(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules, "basics": BASICS}).as_object().unwrap())
}

/// The round intent timeline with AI tactics and ground hazards on (`on`), or the same without `ai_tactics`.
fn intent_rules(on: bool) -> Ruleset {
    let mut rules = vec![ROUND_INTENT, ENEMY_MOVES, AP_COST, GROUND_HAZARDS];
    if on {
        rules.push(AI_TACTICS);
    }
    ruleset(&rules)
}

/// Every unit of the catalog in a battle under `rules`, off the field (tests place the ones they use).
fn session(rules: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_plate", "p_cloth", "p_bow"], "enemies": ["e_flag", "e_bailiff", "e_footman", "e_mate", "e_purifier"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
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

fn v(x: i32, y: i32) -> V2 {
    V2::new(x, y)
}

/// The round intent AI's round for `c` (its own profile, Balanced) on a fresh plan.
fn pick(w: &mut World, c: UnitId) -> (IntentRound, Plan) {
    let mut plan = Plan::new();
    let p = w.profile_of(c);
    let r = ai::plan_round_intent(w, &mut plan, c, &p, Tactic::Balanced);
    (r, plan)
}

fn intent_target(r: &IntentRound) -> Option<(RoundIntent, RoundTarget)> {
    r.pick.as_ref().map(|p| (p.intent, p.target))
}

/// The classic AI's (TacticalAI) draft for `c` on a fresh plan.
fn classic_steps(w: &World, c: UnitId) -> Vec<Step> {
    let mut plan = Plan::new();
    ai::build_steps(w, &mut plan, c, &w.profile_of(c))
}

/// The first single-target attack step's target in a classic draft.
fn classic_target(w: &World, c: UnitId) -> Option<UnitId> {
    classic_steps(w, c).iter().find(|s| s.mode == Mode::Engage).and_then(|s| s.target)
}

/// A walkable terrain given ranged cover 0.4 (its index).
fn cover_terrain(w: &mut World) -> usize {
    let t = TERRAIN_IDS.iter().position(|t| *t == "grass").unwrap();
    w.grid.table.rows[t].cover_guard = 0.4;
    t
}

fn set_terrain(w: &mut World, cell: V2, t: usize) {
    w.grid.terrain[(cell.y * rpgkit_battle::grid::GRID_W + cell.x) as usize] = t;
}

fn wall(w: &mut World, cell: V2) {
    let t = TERRAIN_IDS.iter().position(|t| *t == "wall").unwrap();
    set_terrain(w, cell, t);
}

fn lay(w: &mut World, id: &str, by: UnitId, cell: V2) {
    let def = w.cat.hazards[id].clone();
    w.grid.ground.insert(cell, Placed { def, by, round: 1, last_round: 3 });
}

// ---- AT1: target preferences ----------------------------------------------------------------------------------------

/// SOFTEST: of two foes the Flagellant reaches this round it takes the lower armour rating (armour 6 over 25), though
/// the armoured one stands nearer; one out of reach this round ranks last; a reachable taunter ranks first. The
/// classic AI ranks the same way. Rule off: SOFTEST reads as ROLE (an Aggressor: nearest first).
#[test]
fn softest_takes_the_lowest_armour_rating() {
    let mut s = session(intent_rules(true));
    let w = &mut s.w;
    assert!(damage::armor_rating(&w.units[PLATE]) > damage::armor_rating(&w.units[CLOTH]), "the fixture: plate is armoured");
    assert_eq!(w.profile_of(FLAG).target_pref, TargetPref::Softest);
    place(w, FLAG, v(6, 4));
    place(w, PLATE, v(5, 4));
    place(w, CLOTH, v(8, 4));
    let (r, _) = pick(w, FLAG);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_ASSAULT, Some((RoundIntent::Assault, RoundTarget::Unit(CLOTH)))));
    // a reachable taunter first
    status::grant_now(&mut w.units[PLATE], "taunt", 2, 0, "test");
    assert!(w.units[PLATE].has_status("taunt"));
    let (r, _) = pick(w, FLAG);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(PLATE))), "the taunter");
    w.units[PLATE].remove_status("taunt");
    // out of reach this round: last, behind the armoured foe in reach
    place(w, FLAG, v(11, 4));
    place(w, PLATE, v(10, 4));
    place(w, CLOTH, v(0, 0));
    let plan = Plan::new();
    assert!(w.pos(FLAG).dist(w.pos(CLOTH)) > plan.max_move_tiles(w, FLAG) + 1, "the fixture: cloth is out of reach this round");
    let (r, _) = pick(w, FLAG);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(PLATE))));
    // the classic AI reads the same ranking
    let mut c = session(ruleset(&[AI_TACTICS]));
    place(&mut c.w, FLAG, v(6, 4));
    place(&mut c.w, PLATE, v(5, 4));
    place(&mut c.w, CLOTH, v(7, 4));
    assert_eq!(classic_target(&c.w, FLAG), Some(CLOTH));
    // rule off: ROLE (nearest)
    let mut off = session(intent_rules(false));
    place(&mut off.w, FLAG, v(6, 4));
    place(&mut off.w, PLATE, v(5, 4));
    place(&mut off.w, CLOTH, v(8, 4));
    let (r, _) = pick(&mut off.w, FLAG);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(PLATE))));
}

/// TOUGHEST: the Bailiff takes the melee foe (230 max HP) over a nearer ranged one with more (250); among melee foes
/// the most max HP. Rule off: ROLE (nearest, the bow).
#[test]
fn toughest_takes_the_melee_foe_with_the_most_max_hp() {
    let mut s = session(intent_rules(true));
    let w = &mut s.w;
    assert!(w.units[BOW].max_hp > w.units[PLATE].max_hp && w.units[BOW].attack_range > 1, "the fixture");
    place(w, BAILIFF, v(6, 4));
    place(w, BOW, v(5, 4));
    place(w, PLATE, v(8, 4));
    let (r, _) = pick(w, BAILIFF);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(PLATE))));
    // two melee foes: the most max HP, though farther
    w.units[CLOTH].max_hp = w.units[PLATE].max_hp + 40;
    w.units[CLOTH].hp = w.units[CLOTH].max_hp;
    place(w, CLOTH, v(6, 6));
    place(w, PLATE, v(7, 4));
    let (r, _) = pick(w, BAILIFF);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(CLOTH))));
    let mut off = session(intent_rules(false));
    place(&mut off.w, BAILIFF, v(6, 4));
    place(&mut off.w, BOW, v(5, 4));
    place(&mut off.w, PLATE, v(8, 4));
    let (r, _) = pick(&mut off.w, BAILIFF);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(BOW))), "ROLE: nearest");
}

/// A phase `ai` effect's `tactics` replaces the unit's list (absent: the list stays); its SOFTEST and TOUGHEST are kept.
#[test]
fn a_phase_ai_effect_replaces_the_tactics() {
    let s = session(intent_rules(true));
    let mut p: AiProfile = s.w.profile_of(FOOTMAN);
    assert_eq!(p.tactics, vec![UnitTactic::BraceAfterHit, UnitTactic::HoldLine]);
    let fx = |v: Value| v.as_object().unwrap().clone();
    rpgkit_battle::encounter::apply_ai(&mut p, &fx(json!({"type": "ai", "unit": "e_footman", "reckless": true})));
    assert_eq!(p.tactics.len(), 2, "no `tactics` key: the list stays");
    rpgkit_battle::encounter::apply_ai(&mut p, &fx(json!({"type": "ai", "unit": "e_footman", "tactics": ["flush_cover"], "target_pref": "TOUGHEST"})));
    assert_eq!((p.tactics.clone(), p.target_pref), (vec![UnitTactic::FlushCover], TargetPref::Toughest));
    rpgkit_battle::encounter::apply_ai(&mut p, &fx(json!({"type": "ai", "unit": "e_footman", "tactics": []})));
    assert!(p.tactics.is_empty());
}

// ---- AT2: brace_after_hit, hold_line ------------------------------------------------------------------------------

/// `brace_after_hit`: a friend within 2 tiles (or the Footman itself) lost HP to an attack hit last round: Guard (the
/// row before row 4); a friend 3 tiles off, nobody hit, a reckless Footman or the rule off: the table (Assault).
#[test]
fn brace_after_hit_guards_after_a_friend_was_hit() {
    let mut s = session(intent_rules(true));
    let w = &mut s.w;
    // without hold_line, so the Guard is on itself
    w.profiles.get_mut(&FOOTMAN).unwrap().tactics = vec![UnitTactic::BraceAfterHit];
    place(w, FOOTMAN, v(6, 4));
    place(w, MATE, v(7, 5));
    place(w, PLATE, v(4, 4));
    let (r, _) = pick(w, FOOTMAN);
    assert_eq!(r.row, ai::ROW_ASSAULT, "nobody hit: the table");
    w.hits.last_round = vec![MATE];
    let (r, plan) = pick(w, FOOTMAN);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_BRACE, Some((RoundIntent::Guard, RoundTarget::Myself))));
    assert!(plan.entries_for(w, FOOTMAN).iter().any(|e| w.cat.skills[e.skill].id == "active_guard"));
    w.hits.last_round = vec![FOOTMAN];
    assert_eq!(pick(w, FOOTMAN).0.row, ai::ROW_BRACE, "itself hit");
    w.hits.last_round = vec![MATE];
    place(w, MATE, v(8, 5));
    assert_eq!(pick(w, FOOTMAN).0.row, ai::ROW_ASSAULT, "a friend 3 tiles off");
    place(w, MATE, v(7, 5));
    w.profiles.get_mut(&FOOTMAN).unwrap().reckless = true;
    assert_ne!(pick(w, FOOTMAN).0.row, ai::ROW_BRACE, "reckless: no Guard option");
    w.profiles.get_mut(&FOOTMAN).unwrap().reckless = false;
    // rule off
    let mut off = session(intent_rules(false));
    place(&mut off.w, FOOTMAN, v(6, 4));
    place(&mut off.w, MATE, v(7, 5));
    place(&mut off.w, PLATE, v(4, 4));
    off.w.hits.last_round = vec![MATE];
    assert_eq!(pick(&mut off.w, FOOTMAN).0.row, ai::ROW_ASSAULT);
}

/// `brace_after_hit` with `hold_line`: the Guard is on the hold tile (one move block, next to a friend), not its own.
#[test]
fn brace_with_hold_line_guards_beside_a_friend() {
    let mut s = session(intent_rules(true));
    let w = &mut s.w;
    place(w, FOOTMAN, v(8, 4));
    place(w, MATE, v(9, 5));
    place(w, PLATE, v(0, 0));
    w.hits.last_round = vec![MATE];
    let plan = Plan::new();
    let tile = ai::hold_tile(w, &plan, FOOTMAN).expect("a hold tile");
    assert_eq!(tile.dist(w.pos(MATE)), 1, "next to the friend: {tile}");
    assert!(w.pos(FOOTMAN).dist(tile) <= w.units[FOOTMAN].move_per_block(), "one move block");
    let (r, _) = pick(w, FOOTMAN);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_BRACE, Some((RoundIntent::Guard, RoundTarget::Cell(tile)))));
}

/// The resolver records who lost HP to an attack hit and the round's end makes it last round's (rule on); nothing is
/// recorded without the rule.
#[test]
fn hits_are_remembered_for_one_round() {
    let mut w = session(ruleset(&[AI_TACTICS])).w;
    w.note_hits(&[Struck { target: MATE, damage: 5, ranged: false }, Struck { target: FLAG, damage: 0, ranged: true }]);
    assert_eq!(w.hits.this_round, vec![MATE], "a hit that took no HP does not count");
    w.roll_hits();
    assert_eq!((w.hits.this_round.clone(), w.hits.last_round.clone()), (Vec::new(), vec![MATE]));
    w.roll_hits();
    assert!(w.hits.last_round.is_empty());
    // a session round: the plate's jab lands on the mate
    let mut s = session(ruleset(&[AI_TACTICS]));
    place(&mut s.w, PLATE, v(4, 4));
    place(&mut s.w, MATE, v(5, 4));
    let jab = s.w.book().find("t_jab").unwrap();
    assert_eq!(s.plan.add_step(&s.w, PLATE, rpgkit_battle::plan::PlanStep::engage(jab, MATE), true), "");
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
    let h = &s.w.hits;
    assert!(s.w.units[MATE].hp < s.w.units[MATE].max_hp && (h.this_round.contains(&MATE) || h.last_round.contains(&MATE)), "{h:?}");
    let mut off = session(ruleset(&[]));
    place(&mut off.w, PLATE, v(4, 4));
    place(&mut off.w, MATE, v(5, 4));
    assert_eq!(off.plan.add_step(&off.w, PLATE, rpgkit_battle::plan::PlanStep::engage(jab, MATE), true), "");
    while off.current_round == 1 && !off.combat_over && off.phase != Phase::Replay {
        off.step_next();
    }
    assert!(off.w.units[MATE].hp < off.w.units[MATE].max_hp);
    assert_eq!(off.w.hits, Default::default(), "rule off: nothing recorded");
}

/// `hold_line` in PL4: after threat, a tile next to a friend first, then fewer walkable free neighbours; a unit without
/// the tactic ranks as before (here: the lower cell index).
#[test]
fn hold_line_ranks_tiles_beside_a_friend() {
    let mut s = session(intent_rules(true));
    let w = &mut s.w;
    place(w, FOOTMAN, v(5, 4));
    place(w, MATE, v(8, 4));
    place(w, PLATE, v(0, 8));
    let (open, beside) = (v(5, 3), v(7, 4));
    assert_eq!(round_intent::threat(w, FOOTMAN, open), round_intent::threat(w, FOOTMAN, beside), "the fixture: equal threat");
    let cands = || vec![(open, vec![open]), (beside, vec![v(6, 4), beside])];
    let mut a = cands();
    round_intent::rank_tiles(w, FOOTMAN, w.pos(PLATE), false, Aim::Toward, &mut a);
    assert_eq!(a[0].0, beside, "next to a friend");
    // two tiles beside the friend: fewer free neighbours first (a wall beside one)
    let (b1, b2) = (v(8, 3), v(8, 5));
    wall(w, v(9, 5));
    let mut b = vec![(b1, vec![b1]), (b2, vec![b2])];
    round_intent::rank_tiles(w, FOOTMAN, w.pos(PLATE), false, Aim::Toward, &mut b);
    assert_eq!(b[0].0, b2, "the tile with fewer free neighbours");
    // without the tactic: as before
    w.profiles.get_mut(&FOOTMAN).unwrap().tactics.clear();
    let mut c = cands();
    round_intent::rank_tiles(w, FOOTMAN, w.pos(PLATE), false, Aim::Toward, &mut c);
    assert_eq!(c[0].0, open, "fewer path tiles");
}

// ---- AT2: flush_cover -----------------------------------------------------------------------------------------------

/// The Purifier (reach 2-4 PLUS) at (1, 4); its first ranked target A at (3, 4) (the weakest), B at (4, 4) in the
/// open, C at (2, 5) on a tile with cover. Aims holding A: A's own (A, B: 2), (3, 5) (A, C: 2, + 1 for C in cover).
fn flush_board(w: &mut World) -> V2 {
    let t = cover_terrain(w);
    place(w, PURIFIER, v(1, 4));
    place(w, PLATE, v(3, 4));
    place(w, CLOTH, v(4, 4));
    place(w, BOW, v(2, 5));
    set_terrain(w, v(2, 5), t);
    w.units[PLATE].hp -= 60;
    assert!(rpgkit_battle::command::cover_for(&w.grid, v(1, 4), v(2, 5), true) > 0.0, "the fixture: C in cover");
    v(3, 5)
}

/// Round intent: `flush_cover` aims the area opener at the tile that also catches the foe in cover (a tile target, the
/// blast its opener); without the cover, or without the tactic, the opener aims at the first ranked target.
#[test]
fn flush_cover_aims_the_area_opener_at_cover() {
    let mut s = session(intent_rules(true));
    let w = &mut s.w;
    let aim = flush_board(w);
    let (r, plan) = pick(w, PURIFIER);
    assert_eq!((r.row, intent_target(&r)), (ai::ROW_ASSAULT, Some((RoundIntent::Assault, RoundTarget::Cell(aim)))));
    let area: Vec<V2> = plan.steps_of(PURIFIER).iter().filter(|s| s.mode == Mode::Area).map(|s| s.cell.get()).collect();
    assert_eq!(area.first(), Some(&aim), "{area:?}");
    // the cover gone: the plain pick
    set_terrain(w, v(2, 5), 0);
    let (r, _) = pick(w, PURIFIER);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(PLATE))));
    // no tactic: the plain pick
    let mut no = session(intent_rules(true));
    flush_board(&mut no.w);
    no.w.profiles.get_mut(&PURIFIER).unwrap().tactics.clear();
    let (r, _) = pick(&mut no.w, PURIFIER);
    assert_eq!(intent_target(&r), Some((RoundIntent::Assault, RoundTarget::Unit(PLATE))));
}

/// Classic AI: the area aim's score counts a foe hit in cover once more, so the aim catching C wins over the tie the
/// classic order would break toward A's own row; without the rule the classic aim.
#[test]
fn flush_cover_in_the_classic_area_aim() {
    let first_area = |w: &World| classic_steps(w, PURIFIER).iter().find(|s| s.mode == Mode::Area).map(|s| s.cell.get());
    let mut on = session(ruleset(&[AI_TACTICS]));
    let aim = flush_board(&mut on.w);
    assert_eq!(first_area(&on.w), Some(aim));
    let mut off = session(ruleset(&[]));
    flush_board(&mut off.w);
    let plain = first_area(&off.w);
    assert!(plain.is_some() && plain != Some(aim), "{plain:?}");
}

// ---- AT3: hazard-aware tiles ----------------------------------------------------------------------------------------

/// PL4's threat adds 1 for a ground hazard that would harm the unit and that its side sees: the party's fire does, the
/// party's hidden tripwire does not (not seen), smoke does not (harmless); rule off: no hazard threat.
#[test]
fn pl4_threat_counts_a_visible_harmful_hazard() {
    let mut s = session(intent_rules(true));
    let w = &mut s.w;
    place(w, FOOTMAN, v(6, 4));
    place(w, PLATE, v(0, 0));
    let (fire, trip, smoke, bare) = (v(7, 4), v(6, 5), v(5, 4), v(6, 3));
    let base: Vec<usize> = [fire, trip, smoke, bare].iter().map(|c| round_intent::threat(w, FOOTMAN, *c)).collect();
    lay(w, "g_fire", PLATE, fire);
    lay(w, "g_trip", PLATE, trip);
    lay(w, "g_smoke", PLATE, smoke);
    let now: Vec<usize> = [fire, trip, smoke, bare].iter().map(|c| round_intent::threat(w, FOOTMAN, *c)).collect();
    assert_eq!(now, vec![base[0] + 1, base[1], base[2], base[3]]);
    let mut a = vec![(fire, vec![fire]), (bare, vec![bare])];
    round_intent::rank_tiles(w, FOOTMAN, w.pos(PLATE), false, Aim::Toward, &mut a);
    assert_eq!(a[0].0, bare, "PL4 avoids the fire");
    let mut off = session(intent_rules(false));
    place(&mut off.w, FOOTMAN, v(6, 4));
    place(&mut off.w, PLATE, v(0, 0));
    let before = round_intent::threat(&off.w, FOOTMAN, fire);
    lay(&mut off.w, "g_fire", PLATE, fire);
    assert_eq!(round_intent::threat(&off.w, FOOTMAN, fire), before, "rule off: no hazard threat");
}

// ---- 4.1, 4.2: rule off -------------------------------------------------------------------------------------------

/// With the rule off a unit with SOFTEST, TOUGHEST and every tactic plans exactly as the same unit without them, in
/// both AIs (classic draft and round intent pick), with a friend hit last round and a hazard on the board.
#[test]
fn rule_off_plans_as_without_the_fields() {
    let board = |w: &mut World| {
        place(w, FLAG, v(6, 4));
        place(w, FOOTMAN, v(7, 5));
        place(w, PURIFIER, v(9, 4));
        place(w, PLATE, v(5, 4));
        place(w, CLOTH, v(4, 6));
        place(w, BOW, v(2, 2));
        lay(w, "g_fire", PLATE, v(5, 5));
        w.hits.last_round = vec![FOOTMAN];
    };
    let strip = |w: &mut World| {
        for c in [FLAG, BAILIFF, FOOTMAN, PURIFIER] {
            let p = w.profiles.get_mut(&c).unwrap();
            p.tactics.clear();
            p.target_pref = p.role_pref();
        }
    };
    for rules in [intent_rules(false), ruleset(&[GROUND_HAZARDS])] {
        let mut a = session(rules.clone());
        let mut b = session(rules);
        board(&mut a.w);
        board(&mut b.w);
        strip(&mut b.w);
        for c in [FLAG, FOOTMAN, PURIFIER] {
            let steps = |w: &World| classic_steps(w, c).iter().map(|s| format!("{:?}", s)).collect::<Vec<_>>();
            assert_eq!(steps(&a.w), steps(&b.w), "classic {c}");
            let (ra, pa) = pick(&mut a.w, c);
            let (rb, pb) = pick(&mut b.w, c);
            assert_eq!((ra.row, intent_target(&ra)), (rb.row, intent_target(&rb)), "intent {c}");
            assert_eq!(format!("{:?}", pa.steps_of(c)), format!("{:?}", pb.steps_of(c)));
        }
    }
}
