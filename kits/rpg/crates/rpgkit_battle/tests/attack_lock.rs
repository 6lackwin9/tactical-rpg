//! ATTACKLOCK1 (decision "Attack lock at wind-up") on a scratch catalog built here (no shipped data): under `attack_lock`
//! a melee attack commits to its target's tile when its wind-up starts, after any chase or dash, and misses at impact if
//! the target has left it (`[Dodged]`, AP spent); each swing commits again; ranged attacks hit in band, area attacks keep
//! their tiles, and classic and `none` hit as before. The tick order: a block starts every action due in `scheduled`
//! order (the lock is taken there), then the block's impacts land by initiative (a Walk lands there), so a Walk landing
//! on an attack's start block leaves after the lock, one landing before it is chased, and a Walk on a one-block attack's
//! impact block goes by initiative. Decision "Attack lock answers": a swing locks only a target in reach at its start; a
//! forced move carries the lock along; a same-tick dash leaves the tile for both sides; a dodge raises `evaded`.

use indexmap::IndexMap;
use rpgkit_battle::arena;
use rpgkit_battle::command::{CmdKind, Command, MoveKind};
use rpgkit_battle::grid::{V2, NO_CELL};
use rpgkit_battle::log;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, ManeuverKind, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{AttackLock, Ruleset, AP_COST, ARENA_HAZARDS, FLANK, FORCED_MOVE, ROUND_INTENT, TIMELINE, TRIGGERS};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const ARCHER: UnitId = 2;
const FOE: UnitId = 3;
const FOE_B: UnitId = 4;

/// Engine skills and the test kit: melee attacks of one, two and three blocks, a Maneuver attack with a position, a band
/// (1, 2) as a reach weapon gives, a band (1, 3) that reaches past it, a ranged shot and an area.
fn catalog() -> Catalog {
    let atk = |id: &str, blocks: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": blocks, "ap_cost": 2, "power_mult": 1.0, "target_mode": "ENEMY"});
        row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        row
    };
    let skills = vec![
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        atk("tl_cut", 1, json!({})),
        atk("tl_heavy", 2, json!({"ap_cost": 3})),
        atk("tl_long", 3, json!({})),
        atk("tl_slip_strike", 2, json!({"round_intent": "maneuver", "position": "flank", "reach_max": 2})),
        atk("flank_cut", 1, json!({"round_intent": "maneuver", "position": "flank", "reach_max": 2})),
        atk("tl_thrust", 2, json!({"reach_min": 1, "reach_max": 2})),
        atk("tl_hook", 2, json!({"reach_min": 1, "reach_max": 3})),
        atk("tl_aimed_shot", 2, json!({})),
        atk("tl_burst", 2, json!({"area": "BURST", "reach_min": 1, "reach_max": 3})),
        atk("tl_shove", 1, json!({"forced_move": {"kind": "push", "tiles": 1}})),
        json!({"id": "tl_hook_guard", "name": "tl_hook_guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3,
            "triggers": [{"on": "ally_hit", "when": {"range": 1, "melee": true}, "effects": [{"type": "redirect", "pct": 100}, {"type": "move", "kind": "pull", "tiles": 1, "to": "other"}], "limit": "round"}]}),
    ];
    let unit = |id: &str, side: &str, range: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": range, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("tl_vanguard", "party", 1), unit("tl_ally", "party", 1), unit("tl_archer", "party", 3), unit("tl_footman", "enemy", 1), unit("tl_footman_b", "enemy", 1)]}))
}

/// A timeline row with the round intent, `ap_cost`, the flank rule and what moves a unit by force; `lock` as `attack_lock`
/// (a row value).
fn timeline(lock: &str) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK, FORCED_MOVE, TRIGGERS, ARENA_HAZARDS], "attack_lock": lock}).as_object().unwrap())
}

/// The board: tl_vanguard (4, 4) next to tl_footman (5, 4); the others away. No planned round (tests book every action).
fn world(r: Ruleset) -> World {
    session(r, json!([])).w
}

fn session(r: Ruleset, hazards: Value) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["tl_vanguard", "tl_ally", "tl_archer"], "enemies": ["tl_footman", "tl_footman_b"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337, "arena_hazards": hazards});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = r;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(0, 8)), (ARCHER, V2::new(0, 0)), (FOE, V2::new(5, 4)), (FOE_B, V2::new(11, 8))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn hit(w: &World, c: UnitId, t: UnitId, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, PlanStep::engage(sid(w, skill), t), true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

/// `c` takes Maneuver Engage on `t`: its attacks with a position carry their dash.
fn engage(w: &mut World, c: UnitId, t: UnitId) {
    round_intent::set_intent(w, c, RoundIntent::Maneuver).unwrap();
    round_intent::set_target(w, c, RoundTarget::Unit(t)).unwrap();
    round_intent::set_maneuver(w, c, ManeuverKind::Engage).unwrap();
}

/// A Walk of the given one-tile steps, landing in its one block.
fn walk(c: UnitId, dirs: &[V2]) -> Command {
    let mut cmd = Command::new(c, 0, 1, CmdKind::Walk { path: dirs.to_vec(), expected_start: NO_CELL, goal: NO_CELL, approach: None, moved_tiles: 0, cut_short: false, blocked_by: String::new() });
    cmd.name = "Move".into();
    cmd
}

fn v(x: i32, y: i32) -> V2 {
    V2::new(x, y)
}

fn book(res: &mut Resolver, c: UnitId, cmd: Command, start: i32) -> usize {
    assert!(res.schedule(c, cmd, start), "booked at {start}");
    res.scheduled.len() - 1
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

fn dodged(res: &Resolver, a: usize, t: UnitId) -> bool {
    let cmd = &res.scheduled[a].command;
    let on = cmd.dodged == Some(t);
    assert_eq!(on, cmd.log.starts_with("[Dodged]"), "the field and the line agree: {}", cmd.log);
    if on {
        assert_eq!(cmd.evaded, Some(t), "a dodge raises `evaded`");
        assert!(cmd.failed && cmd.hits.is_empty() && cmd.struck.is_empty() && cmd.reactions.is_empty(), "a dodge is no hit: {}", cmd.log);
    }
    on
}

/// A Move in the wind-up dodges, even to a tile next to the attacker; nothing lands and the AP stays spent (the same AP
/// a hit pays), and the log cell says DODGED.
#[test]
fn a_move_in_the_wind_up_dodges() {
    let mut w = world(timeline("melee"));
    let (hp, ap) = (w.units[FOE].hp, w.units[HERO].ap);
    let cmd = hit(&w, HERO, FOE, "tl_heavy");
    let paid = cmd.ap_cost + cmd.stance_switch_cost;
    assert!(paid > 0);
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    book(&mut res, FOE, walk(FOE, &[v(0, -1), v(-1, 0)]), 1);
    run(&mut w, &mut res, 1, 2);
    assert_eq!(w.pos(FOE), v(4, 3), "next to the attacker");
    assert_eq!(res.scheduled[a].command.locked, Some((FOE, v(5, 4))));
    assert!(dodged(&res, a, FOE), "{:?}", res.log);
    assert_eq!(w.units[FOE].hp, hp, "no damage");
    assert_eq!(w.units[HERO].ap, ap - paid, "AP spent");
    assert_eq!(log::from_action(&w, &res.scheduled[a], 1, 2).outcome, "DODGED");
    assert!(res.log.iter().any(|l| l.starts_with("[Dodged] tl_footman")), "{:?}", res.log);
}

/// A Move that lands before the wind-up starts is chased and hit: the lock takes the tile it stands on then.
#[test]
fn a_move_before_the_wind_up_is_chased_and_hit() {
    let mut w = world(timeline("melee"));
    let hp = w.units[FOE].hp;
    let cmd = hit(&w, HERO, FOE, "tl_heavy");
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
    let a = book(&mut res, HERO, cmd, 2);
    run(&mut w, &mut res, 1, 3);
    assert_eq!(w.pos(HERO), v(5, 4), "the chase");
    assert_eq!(res.scheduled[a].command.moved, Some(MoveKind::Chase));
    assert_eq!(res.scheduled[a].command.locked, Some((FOE, v(6, 4))));
    assert!(!dodged(&res, a, FOE));
    assert!(w.units[FOE].hp < hp, "{:?}", res.log);
}

/// The tick order, both sides: a Walk landing on the block the wind-up starts leaves after the lock (a dodge), one
/// landing the block before is chased (a hit); on a one-block attack's own block the higher initiative goes first (the
/// speed here: a faster Walk dodges, a slower one is hit and then walks).
#[test]
fn the_tick_order_decides_a_tie() {
    for (walk_at, attack_at, hits) in [(1, 2, true), (2, 2, false), (3, 2, false)] {
        let mut w = world(timeline("melee"));
        let hp = w.units[FOE].hp;
        let cmd = hit(&w, HERO, FOE, "tl_heavy");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, attack_at);
        book(&mut res, FOE, walk(FOE, &[v(1, 0)]), walk_at);
        w.units[FOE].speed = w.units[HERO].speed + 1;
        run(&mut w, &mut res, 1, 3);
        assert_eq!(w.units[FOE].hp < hp, hits, "walk at {walk_at}, attack at {attack_at}: {:?}", res.log);
        assert_eq!(dodged(&res, a, FOE), !hits, "walk at {walk_at}, attack at {attack_at}");
    }
    for (walker_faster, hits) in [(true, false), (false, true)] {
        let mut w = world(timeline("melee"));
        let hp = w.units[FOE].hp;
        w.units[FOE].speed = w.units[HERO].speed + if walker_faster { 1 } else { -1 };
        let cmd = hit(&w, HERO, FOE, "tl_cut");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.pos(FOE), v(6, 4), "the Walk lands either way");
        assert_eq!(w.units[FOE].hp < hp, hits, "walker faster {walker_faster}: {:?}", res.log);
        assert_eq!(dodged(&res, a, FOE), !hits, "walker faster {walker_faster}");
    }
}

/// A Maneuver dash to the attacker's flank dodges, though it ends next to the attacker. On the wind-up's own block it
/// dodges too (decision "Attack lock answers" 4), whichever side swings and whichever was scheduled first.
#[test]
fn a_maneuver_dash_to_the_flank_dodges() {
    for (swinger, dasher) in [(HERO, FOE), (FOE, HERO)] {
        for (dash_at, dash_first) in [(2, false), (1, false), (1, true)] {
            let mut w = world(timeline("melee"));
            round_intent::set_intent(&mut w, swinger, RoundIntent::Assault).unwrap();
            round_intent::set_target(&mut w, swinger, RoundTarget::Unit(dasher)).unwrap();
            engage(&mut w, dasher, swinger);
            let (hp, start) = (w.units[dasher].hp, w.pos(dasher));
            let heavy = hit(&w, swinger, dasher, "tl_heavy");
            let dash = hit(&w, dasher, swinger, "flank_cut");
            let mut res = Resolver::new(BLOCKS, 1);
            let a = if dash_first {
                book(&mut res, dasher, dash, dash_at);
                book(&mut res, swinger, heavy, 1)
            } else {
                let a = book(&mut res, swinger, heavy, 1);
                book(&mut res, dasher, dash, dash_at);
                a
            };
            run(&mut w, &mut res, 1, 2);
            let case = format!("swinger {swinger}, dash at {dash_at}, first {dash_first}: {:?}", res.log);
            assert_ne!(w.pos(dasher), start, "the dash moved it: {case}");
            assert_eq!(w.pos(dasher).dist(w.pos(swinger)), 1, "it ends next to the attacker: {case}");
            assert_eq!(res.scheduled[a].command.locked, Some((dasher, start)), "{case}");
            assert_eq!(w.units[dasher].hp, hp, "{case}");
            assert!(dodged(&res, a, dasher), "{case}");
        }
    }
}

/// The attacker's own Maneuver: its dash happens at the wind-up's tick and the lock comes after it, on the tile its
/// target stands on; the target staying is hit, the target walking in the wind-up dodges.
#[test]
fn a_maneuver_attack_dashes_then_locks() {
    for moves in [false, true] {
        let mut w = world(timeline("melee"));
        w.grid.positions[ALLY] = Some(v(6, 4));
        w.grid.positions[HERO] = Some(v(3, 3));
        round_intent::set_intent(&mut w, FOE, RoundIntent::Assault).unwrap();
        round_intent::set_target(&mut w, FOE, RoundTarget::Unit(ALLY)).unwrap();
        engage(&mut w, HERO, FOE);
        let hp = w.units[FOE].hp;
        let cmd = hit(&w, HERO, FOE, "tl_slip_strike");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(res.scheduled[a].command.moved, Some(MoveKind::Dash), "{:?}", res.log);
        assert_eq!(w.pos(HERO).dist(v(5, 4)), 1, "the dash lands next to the target");
        assert_eq!(res.scheduled[a].command.locked, Some((FOE, v(5, 4))));
        if moves {
            book(&mut res, FOE, walk(FOE, &[v(0, 1)]), 2);
            w.units[FOE].speed = w.units[HERO].speed + 1;
        }
        run(&mut w, &mut res, 2, 2);
        assert_eq!(dodged(&res, a, FOE), moves, "{:?}", res.log);
        assert_eq!(w.units[FOE].hp < hp, !moves);
    }
}

/// Ranged under `melee`: no tile lock: a target that walks but stays in band is hit, one that walks out of band is out of
/// range (not a dodge). Under `all` the ranged attack locks too. A band (1, 3) reaches past melee and reads as ranged.
#[test]
fn ranged_attacks_hit_in_band_unless_the_option_says_all() {
    for (lock, dir, hits, out, dodge) in [("melee", 1, true, false, false), ("melee", -1, false, true, false), ("all", 1, false, false, true), ("none", 1, true, false, false)] {
        let mut w = world(timeline(lock));
        w.grid.positions[HERO] = Some(v(0, 8));
        w.grid.positions[ALLY] = Some(v(1, 8));
        w.grid.positions[ARCHER] = Some(v(2, 4));
        let hp = w.units[FOE].hp;
        let cmd = hit(&w, ARCHER, FOE, "tl_aimed_shot");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, ARCHER, cmd, 1);
        // dir 1: one step closer (distance 2, in the band 2-3); dir -1: one step away (distance 4)
        book(&mut res, FOE, walk(FOE, &[v(-dir, 0)]), 1);
        run(&mut w, &mut res, 1, 2);
        let case = format!("{lock}, dir {dir}: {:?}", res.log);
        assert_eq!(w.units[FOE].hp < hp, hits, "{case}");
        assert_eq!(res.scheduled[a].command.log.starts_with("[Attack Out of Range]"), out, "{case}");
        assert_eq!(dodged(&res, a, FOE), dodge, "{case}");
    }
    for (lock, dodge) in [("melee", false), ("all", true)] {
        let mut w = world(timeline(lock));
        let hp = w.units[FOE].hp;
        let cmd = hit(&w, HERO, FOE, "tl_hook");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
        run(&mut w, &mut res, 1, 2);
        assert_eq!(dodged(&res, a, FOE), dodge, "{lock}");
        assert_eq!(w.units[FOE].hp < hp, !dodge, "{lock}");
    }
}

/// A reach band (1, 2) counts as melee: a target that steps within the band still dodges; one that stays is hit from 2.
#[test]
fn a_reach_band_counts_as_melee() {
    assert!(round_intent::melee_band((1, 1)) && round_intent::melee_band((1, 2)));
    assert!(!round_intent::melee_band((1, 3)) && !round_intent::melee_band((2, 3)) && !round_intent::melee_band((2, 2)));
    for moves in [false, true] {
        let mut w = world(timeline("melee"));
        w.grid.positions[FOE] = Some(v(6, 4));
        let hp = w.units[FOE].hp;
        let cmd = hit(&w, HERO, FOE, "tl_thrust");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        if moves {
            book(&mut res, FOE, walk(FOE, &[v(-1, 0)]), 1);
        }
        run(&mut w, &mut res, 1, 2);
        assert_eq!(dodged(&res, a, FOE), moves, "{:?}", res.log);
        assert_eq!(w.units[FOE].hp < hp, !moves);
    }
}

/// Area attacks are the same under every value: no lock, the tiles fixed at planning: a unit on them is hit, one that
/// walked off is not, and that is an area miss, never a dodge.
#[test]
fn area_attacks_keep_their_tiles() {
    let mut seen: Vec<(i32, String)> = Vec::new();
    for lock in ["none", "melee", "all"] {
        for moves in [false, true] {
            let mut w = world(timeline(lock));
            round_intent::set_intent(&mut w, HERO, RoundIntent::Assault).unwrap();
            round_intent::set_target(&mut w, HERO, RoundTarget::Cell(v(5, 4))).unwrap();
            let mut plan = Plan::new();
            assert_eq!(plan.add_step(&w, HERO, PlanStep::area_at(sid(&w, "tl_burst"), v(5, 4)), true), "");
            let k = sid(&w, "tl_burst");
            let cmd = plan.entries_for(&w, HERO).into_iter().find(|e| e.skill == k).unwrap().build_command(&w, HERO);
            let hp = w.units[FOE].hp;
            let mut res = Resolver::new(BLOCKS, 1);
            let a = book(&mut res, HERO, cmd, 1);
            if moves {
                book(&mut res, FOE, walk(FOE, &[v(1, 0), v(1, 0), v(1, 0)]), 1);
            }
            run(&mut w, &mut res, 1, 2);
            let c = &res.scheduled[a].command;
            assert_eq!(c.locked, None, "{lock}");
            assert_eq!(c.dodged, None, "{lock}");
            assert_eq!(w.units[FOE].hp < hp, !moves, "{lock}, moves {moves}: {:?}", res.log);
            assert_eq!(c.log.starts_with("[Area Miss]"), moves, "{lock}, moves {moves}: {}", c.log);
            seen.push((w.units[FOE].hp, c.log.clone()));
        }
    }
    assert_eq!(seen[0..2], seen[2..4], "melee as none");
    assert_eq!(seen[0..2], seen[4..6], "all as none");
}

/// Classic and a row at `none` (or a row that does not name the field) hit a target that moved next to the attacker.
#[test]
fn classic_and_none_are_unchanged() {
    let unset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, AP_COST, FLANK]}).as_object().unwrap());
    assert_eq!(unset.attack_lock(), AttackLock::None, "the row's default");
    assert_eq!(Ruleset::classic().attack_lock(), AttackLock::None, "the built-in classic values");
    for r in [Ruleset::classic(), timeline("none"), unset] {
        let id = r.id.clone();
        let mut w = world(r);
        w.units[HERO].active_target = Some(FOE);
        let hp = w.units[FOE].hp;
        let cmd = hit(&w, HERO, FOE, "tl_heavy");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        book(&mut res, FOE, walk(FOE, &[v(0, -1), v(-1, 0)]), 1);
        run(&mut w, &mut res, 1, 2);
        assert_eq!(res.scheduled[a].command.locked, None, "{id}");
        assert!(!dodged(&res, a, FOE), "{id}");
        assert!(w.units[FOE].hp < hp, "{id}: {:?}", res.log);
    }
}

/// Each swing commits again and the round target stays: the first swing misses the walked target, the next chases it,
/// locks its new tile and lands.
#[test]
fn each_swing_commits_again() {
    let mut w = world(timeline("melee"));
    round_intent::set_intent(&mut w, HERO, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut w, HERO, RoundTarget::Unit(FOE)).unwrap();
    let hp = w.units[FOE].hp;
    let (first, second) = (hit(&w, HERO, FOE, "tl_heavy"), hit(&w, HERO, FOE, "tl_heavy"));
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, first, 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
    let b = book(&mut res, HERO, second, 3);
    run(&mut w, &mut res, 1, 2);
    assert!(dodged(&res, a, FOE));
    assert_eq!(w.units[FOE].hp, hp);
    assert_eq!(w.round_states[&HERO].target, Some(RoundTarget::Unit(FOE)), "the round target stays");
    run(&mut w, &mut res, 3, 4);
    assert_eq!(res.scheduled[b].command.moved, Some(MoveKind::Chase));
    assert_eq!(res.scheduled[b].command.locked, Some((FOE, v(6, 4))));
    assert!(!dodged(&res, b, FOE));
    assert!(w.units[FOE].hp < hp, "{:?}", res.log);
}

/// The compare reads the tile at impact: a target that left and came back is hit, and another unit that stepped onto the
/// tile is not struck in its place (the lock is on the unit and its tile).
#[test]
fn the_lock_is_read_at_impact_and_stays_on_its_unit() {
    let mut w = world(timeline("melee"));
    let hp = w.units[FOE].hp;
    let cmd = hit(&w, HERO, FOE, "tl_long");
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
    book(&mut res, FOE, walk(FOE, &[v(-1, 0)]), 2);
    run(&mut w, &mut res, 1, 3);
    assert!(!dodged(&res, a, FOE));
    assert!(w.units[FOE].hp < hp, "back on the tile: {:?}", res.log);

    let mut w = world(timeline("melee"));
    w.grid.positions[FOE_B] = Some(v(5, 5));
    let hp = (w.units[FOE].hp, w.units[FOE_B].hp);
    let cmd = hit(&w, HERO, FOE, "tl_long");
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
    book(&mut res, FOE_B, walk(FOE_B, &[v(0, -1)]), 2);
    run(&mut w, &mut res, 1, 3);
    assert_eq!(w.pos(FOE_B), v(5, 4), "it stepped onto the locked tile");
    assert!(dodged(&res, a, FOE));
    assert_eq!((w.units[FOE].hp, w.units[FOE_B].hp), hp, "nobody is struck");
}

/// A dodge is no hit, so it draws no riposte; the same swing without the lock draws one.
#[test]
fn a_dodge_draws_no_riposte() {
    for (lock, reacts) in [("melee", 0), ("none", 1)] {
        let mut w = world(timeline(lock));
        status::grant_now(&mut w.units[FOE], "riposte", 2, 0, "test");
        let hero_hp = w.units[HERO].hp;
        let cmd = hit(&w, HERO, FOE, "tl_heavy");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        book(&mut res, FOE, walk(FOE, &[v(0, -1), v(-1, 0)]), 1);
        run(&mut w, &mut res, 1, 2);
        assert_eq!(res.scheduled[a].command.reactions.len(), reacts, "{lock}: {:?}", res.log);
        assert_eq!(w.units[HERO].hp < hero_hp, reacts > 0, "{lock}");
    }
}

/// The order at impact: a gone target, then a hidden target (`[Lost]`, the `evaded` field), then a protected flank, then
/// the dodge, then the band; a target that stayed while the attacker was moved off is out of range, not a dodge. TG4 (the
/// target gone before the start) turns the command to Rest, with no lock.
#[test]
fn the_paths_keep_their_order() {
    let setup = |w: &mut World| {
        let cmd = hit(w, HERO, FOE, "tl_heavy");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
        run(w, &mut res, 1, 1);
        (res, a)
    };
    // the target is gone at impact
    let mut w = world(timeline("melee"));
    let (mut res, a) = setup(&mut w);
    let hp = w.units[FOE].hp;
    w.damage(FOE, hp, true);
    run(&mut w, &mut res, 2, 2);
    assert!(res.scheduled.iter().any(|x| x.unit == HERO));
    assert!(res.scheduled[a].command.log.starts_with("[Attack Miss]"), "{}", res.log.join(" | "));
    assert_eq!(res.scheduled[a].command.dodged, None);

    // hidden: stealthed and moved
    let mut w = world(timeline("melee"));
    status::grant_now(&mut w.units[FOE], "stealth", 2, 0, "test");
    let (mut res, a) = setup(&mut w);
    run(&mut w, &mut res, 2, 2);
    let c = &res.scheduled[a].command;
    assert!(c.log.starts_with("[Lost]"), "{}", c.log);
    assert_eq!((c.evaded, c.dodged), (Some(FOE), None), "a lost attack is not a dodge");

    // a protected flank (its miss set at the start) comes before the dodge
    let mut w = world(timeline("melee"));
    let mut cmd = hit(&w, HERO, FOE, "tl_heavy");
    cmd.missed = true;
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
    run(&mut w, &mut res, 1, 2);
    let c = &res.scheduled[a].command;
    assert!(c.log.starts_with("[Flank Protected]"), "{}", c.log);
    assert_eq!(c.dodged, None);
    assert_eq!(log::from_action(&w, &res.scheduled[a], 1, 2).outcome, "FLANK PROTECTED");

    // the target stayed, the attacker was moved off: out of range
    let mut w = world(timeline("melee"));
    let cmd = hit(&w, HERO, FOE, "tl_heavy");
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    w.grid.positions[HERO] = Some(v(1, 4));
    run(&mut w, &mut res, 2, 2);
    let c = &res.scheduled[a].command;
    assert!(c.log.starts_with("[Attack Out of Range]"), "{}", c.log);
    assert_eq!(c.dodged, None);
    assert_eq!(log::from_action(&w, &res.scheduled[a], 1, 2).outcome, "OUT OF RANGE");

    // TG4: the target gone before the start: Rest, no AP, no lock
    let mut w = world(timeline("melee"));
    round_intent::set_intent(&mut w, HERO, RoundIntent::Assault).unwrap();
    round_intent::set_target(&mut w, HERO, RoundTarget::Unit(FOE)).unwrap();
    let cmd = hit(&w, HERO, FOE, "tl_heavy");
    let hp = w.units[FOE].hp;
    w.damage(FOE, hp, true);
    let ap = w.units[HERO].ap;
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 2);
    let c = &res.scheduled[a].command;
    assert!(matches!(c.kind, CmdKind::Rest { .. }));
    assert_eq!((c.locked, c.dodged, w.units[HERO].ap), (None, None, ap));
}

/// Decision "Attack lock answers" 3: a target out of reach after the chase takes no lock, so the band test decides at
/// impact: one that walks into reach is hit, one that stays out is out of range; neither is a dodge.
#[test]
fn a_target_out_of_reach_at_the_start_takes_no_lock() {
    for (step, hits) in [(v(-1, 0), true), (v(0, 1), false)] {
        let mut w = world(timeline("melee"));
        let cmd = hit(&w, HERO, FOE, "tl_heavy");
        w.grid.positions[FOE] = Some(v(7, 4));
        let hp = w.units[FOE].hp;
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        book(&mut res, FOE, walk(FOE, &[step]), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.pos(HERO), v(5, 4), "the chase covers one tile and leaves it out of reach");
        assert_eq!(res.scheduled[a].command.locked, None);
        run(&mut w, &mut res, 2, 2);
        assert!(!dodged(&res, a, FOE), "{:?}", res.log);
        assert_eq!(w.units[FOE].hp < hp, hits, "{:?}", res.log);
        assert_eq!(res.scheduled[a].command.log.starts_with("[Attack Out of Range]"), !hits);
    }
}

/// Decision "Attack lock answers" 1, through `forced::resolve`: an ally's shove does not dodge: the lock follows it, the
/// swing hits while the target is in band and is out of range past it. Only the target's own move dodges: a Walk after the
/// shove leaves the followed tile, and a Walk before it stays a dodge.
#[test]
fn a_shove_carries_the_lock_along() {
    for (skill, hits) in [("tl_thrust", true), ("tl_heavy", false)] {
        let mut w = world(timeline("melee"));
        w.grid.positions[ALLY] = Some(v(5, 3));
        let hp = w.units[FOE].hp;
        let swing = hit(&w, HERO, FOE, skill);
        let shove = hit(&w, ALLY, FOE, "tl_shove");
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, swing, 1);
        book(&mut res, ALLY, shove, 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.pos(FOE), v(5, 5), "{:?}", res.log);
        assert_eq!(res.scheduled[a].command.locked, Some((FOE, v(5, 5))));
        let after_shove = w.units[FOE].hp;
        assert!(after_shove < hp);
        run(&mut w, &mut res, 2, 2);
        assert!(!dodged(&res, a, FOE), "{skill}: {:?}", res.log);
        assert_eq!(w.units[FOE].hp < after_shove, hits, "{skill}: {:?}", res.log);
        assert_eq!(res.scheduled[a].command.log.starts_with("[Attack Out of Range]"), !hits, "{skill}");
    }
    // shove, then its own Walk
    let mut w = world(timeline("melee"));
    w.grid.positions[ALLY] = Some(v(5, 3));
    let (swing, shove) = (hit(&w, HERO, FOE, "tl_long"), hit(&w, ALLY, FOE, "tl_shove"));
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, swing, 1);
    book(&mut res, ALLY, shove, 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 2);
    run(&mut w, &mut res, 1, 3);
    assert!(dodged(&res, a, FOE), "shove then Walk: {:?}", res.log);
    // its own Walk, then a shove: the locked tile is not the one it left, so the lock stays put
    let mut w = world(timeline("melee"));
    w.grid.positions[ALLY] = Some(v(6, 3));
    let (swing, shove) = (hit(&w, HERO, FOE, "tl_long"), hit(&w, ALLY, FOE, "tl_shove"));
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, swing, 1);
    book(&mut res, FOE, walk(FOE, &[v(1, 0)]), 1);
    book(&mut res, ALLY, shove, 2);
    run(&mut w, &mut res, 1, 3);
    assert_eq!(w.pos(FOE), v(6, 5), "{:?}", res.log);
    assert_eq!(res.scheduled[a].command.locked, Some((FOE, v(5, 4))));
    assert!(dodged(&res, a, FOE), "Walk then shove: {:?}", res.log);
}

/// An `intercepting_hook` pull (the trigger effect `move`) carries the lock along: the foe swings at a friend of the
/// holder, is hooked toward the holder, and a third unit's swing locked on it still lands.
#[test]
fn a_hook_pull_carries_the_lock_along() {
    let mut w = world(timeline("melee"));
    w.grid.positions[ALLY] = Some(v(5, 3));
    w.grid.positions[ARCHER] = Some(v(6, 3));
    let hp = w.units[FOE].hp;
    let swing = hit(&w, HERO, FOE, "tl_thrust");
    round_intent::set_target(&mut w, FOE, RoundTarget::Unit(ALLY)).unwrap();
    let cut = hit(&w, FOE, ALLY, "tl_cut");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, ARCHER, PlanStep::self_skill(sid(&w, "tl_hook_guard")), true), "");
    let guard = plan.entries_for(&w, ARCHER)[0].build_command(&w, ARCHER);
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, ARCHER, guard, 1);
    let a = book(&mut res, HERO, swing, 1);
    book(&mut res, FOE, cut, 1);
    run(&mut w, &mut res, 1, 1);
    let pulled = w.pos(FOE);
    assert_ne!(pulled, v(5, 4), "the hook pulled it: {:?}", res.log);
    assert_eq!(res.scheduled[a].command.locked, Some((FOE, pulled)));
    run(&mut w, &mut res, 2, 2);
    assert!(!dodged(&res, a, FOE), "{:?}", res.log);
    assert!(w.units[FOE].hp < hp, "{:?}", res.log);
}

/// An arena push carries the lock along.
#[test]
fn an_arena_push_carries_the_lock_along() {
    let belt = json!({"id": "belt", "name": "Belt", "cells": [[5, 4]], "blocks": [1], "effect": {"push": {"dir": "south", "tiles": 1}}});
    let mut w = session(timeline("melee"), json!([belt])).w;
    let hp = w.units[FOE].hp;
    let swing = hit(&w, HERO, FOE, "tl_thrust");
    let mut res = Resolver::new(BLOCKS, 1);
    for c in 0..w.units.len() {
        res.register(c);
    }
    let a = book(&mut res, HERO, swing, 1);
    run(&mut w, &mut res, 1, 1);
    arena::block_end(&mut res, &mut w, 1, 1, &[]);
    assert_eq!(w.pos(FOE), v(5, 5), "{:?}", res.log);
    assert_eq!(res.scheduled[a].command.locked, Some((FOE, v(5, 5))));
    run(&mut w, &mut res, 2, 2);
    assert!(!dodged(&res, a, FOE), "{:?}", res.log);
    assert!(w.units[FOE].hp < hp, "{:?}", res.log);
}

/// From plan to resolve through a `Session`, the option on: the hero's planned swing misses a foe that walks in its
/// wind-up and lands on one that stays; the ally's planned Move dodges the swing of the other foe. The log cells and the
/// AP read as the rule says.
#[test]
fn a_full_session_plans_and_resolves_a_dodge() {
    for moves in [true, false] {
        let mut s = session(timeline("melee"), json!([]));
        let enemies = s.w.enemies.clone();
        s.resolver.scheduled.retain(|a| !enemies.contains(&a.unit));
        s.w.grid.positions[ALLY] = Some(v(8, 4));
        s.w.grid.positions[FOE_B] = Some(v(9, 4));
        s.set_round_intent(HERO, "assault").unwrap();
        s.set_round_target(HERO, RoundTarget::Unit(FOE)).unwrap();
        let (heavy, step) = (sid(&s.w, "tl_heavy"), sid(&s.w, "move"));
        assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(heavy, FOE), false), "");
        assert_eq!(s.plan.add_step(&s.w, ALLY, PlanStep::mv(step, v(0, 1), 1), false), "");
        let paid = s.plan.entries_for(&s.w, HERO)[0].total_ap();
        let (hp, ap) = (s.w.units[FOE].hp, s.w.units[HERO].ap);
        assert!(s.confirm_plan());
        s.resolver.scheduled.retain(|a| !enemies.contains(&a.unit));
        let foe_b_swing = hit(&s.w, FOE_B, ALLY, "tl_heavy");
        assert!(s.resolver.schedule(FOE_B, foe_b_swing, 1));
        if moves {
            assert!(s.resolver.schedule(FOE, walk(FOE, &[v(1, 0)]), 1));
        }
        let ally_hp = s.w.units[ALLY].hp;
        while s.phase != Phase::Replay {
            s.step_next();
        }
        let of = |u: UnitId, kind: &str| s.log.iter().filter(|e| e.actor == s.w.units[u].name && e.skill == kind).map(|e| e.outcome.clone()).collect::<Vec<_>>();
        assert_eq!(of(HERO, "tl_heavy").contains(&"DODGED".to_string()), moves, "{:?}", s.resolver.log);
        assert_eq!(s.w.units[FOE].hp < hp, !moves, "{:?}", s.resolver.log);
        assert_eq!(s.w.units[HERO].ap, ap - paid, "AP spent either way");
        assert_eq!(of(FOE_B, "tl_heavy"), ["DODGED"], "a planned Move: {:?}", s.resolver.log);
        assert_eq!(s.w.units[ALLY].hp, ally_hp);
    }
}

/// Decision "Attack lock answers" 3 and 4 together: a swing out of reach at its start takes no lock whichever was
/// scheduled first, so a same-tick dash into its reach is hit: the dash starts after the swing's chase.
#[test]
fn a_same_tick_dash_into_reach_is_hit_either_way() {
    for dash_first in [false, true] {
        let mut w = world(timeline("melee"));
        round_intent::set_intent(&mut w, HERO, RoundIntent::Assault).unwrap();
        round_intent::set_target(&mut w, HERO, RoundTarget::Unit(FOE)).unwrap();
        engage(&mut w, FOE, HERO);
        let (heavy, dash) = (hit(&w, HERO, FOE, "tl_heavy"), hit(&w, FOE, HERO, "flank_cut"));
        w.grid.positions[FOE] = Some(v(7, 4));
        let hp = w.units[FOE].hp;
        let mut res = Resolver::new(BLOCKS, 1);
        let a = if dash_first {
            book(&mut res, FOE, dash, 1);
            book(&mut res, HERO, heavy, 1)
        } else {
            let a = book(&mut res, HERO, heavy, 1);
            book(&mut res, FOE, dash, 1);
            a
        };
        run(&mut w, &mut res, 1, 2);
        let case = format!("first {dash_first}: {:?}", res.log);
        assert_eq!(res.scheduled[a].command.locked, None, "{case}");
        assert!(!dodged(&res, a, FOE), "{case}");
        assert!(w.units[FOE].hp < hp, "{case}");
    }
}

/// Two units dashing at each other on one tick: the second reads the tile the first dashed from, so the first dodges it.
/// (The second may need no move: the first's new tile can leave it on a flank tile, MR6, and then the first's swing lands;
/// which unit is first is `scheduled` order, as between any two dashes.)
#[test]
fn a_later_dash_locks_the_tile_its_target_dashed_from() {
    for hero_first in [true, false] {
        let mut w = world(timeline("melee"));
        engage(&mut w, HERO, FOE);
        engage(&mut w, FOE, HERO);
        let (h, f) = (hit(&w, HERO, FOE, "tl_slip_strike"), hit(&w, FOE, HERO, "tl_slip_strike"));
        let mut res = Resolver::new(BLOCKS, 1);
        let (first, second, first_unit, opening) = if hero_first {
            (book(&mut res, HERO, h, 1), book(&mut res, FOE, f, 1), HERO, v(4, 4))
        } else {
            (book(&mut res, FOE, f, 1), book(&mut res, HERO, h, 1), FOE, v(5, 4))
        };
        run(&mut w, &mut res, 1, 2);
        let case = format!("hero first {hero_first}: {:?}", res.log);
        assert_ne!(w.pos(first_unit), opening, "{case}");
        assert_eq!(res.scheduled[second].command.locked, Some((first_unit, opening)), "{case}");
        assert!(dodged(&res, second, first_unit), "{case}");
        assert!(res.scheduled[first].command.locked.is_some(), "{case}");
    }
}
