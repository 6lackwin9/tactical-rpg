//! U1.2-S5 (docs/sessions/U1_STATS.md "Adding a derived value without a logic change", FB-08) on a scratch catalog
//! built here (no shipped data): a pool from data alone, the U1 composure acceptance row. A ruleset's `unit_pools`
//! gives every unit the meter (none without it, nor without the rule unit_meters: the game's off switch); its
//! `max_stat` makes each holder's max its value of the derived row `KNOW x 2 + CONS` clamped to 0..100, `start_full`
//! starts it there; its trigger row (held by every holder) takes 10 when a friend within 2 tiles is knocked; the Rest
//! skill's `meters` gives 5; a band from 1 holds `weaken` below half (a band `from_pct` 50 above it) and a band from 0
//! names the empty pool; a row's `delta_max` bounds one `meter` effect (10 by default). Expectations come from the rows
//! below and those rules.

use indexmap::IndexMap;
use rpgkit_battle::command::Command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::meter;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::bands;
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Unit;
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const POOL: &str = "composure";
/// The derived row's weights and caps (`KNOW x 2 + CONS`, 0..100).
const PER_KNOW: i32 = 2;
const PER_CONS: i32 = 1;
const CAP: i32 = 100;
/// The pool's moves and its low band.
const DOWNED: i32 = -10;
const WITHIN: i32 = 2;
const REST_GAIN: i32 = 5;
const HALF_PCT: i32 = 50;
const LOW_WEAKEN: i32 = 10;

const HERO: UnitId = 0;
const NEAR: UnitId = 1;
const FAR: UnitId = 2;
const ALLY: UnitId = 3;
const FOE: UnitId = 4;

fn catalog(delta: i32, delta_max: Option<i32>) -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2,
            "meters": [{"meter": POOL, "delta": REST_GAIN}]}),
    ];
    let derived = json!({"id": POOL, "name": "Composure", "formula": "KNOW * per_know + CONS * per_cons", "constants": {"per_know": PER_KNOW, "per_cons": PER_CONS}, "min": 0, "max": CAP});
    let mut pool = json!({"id": POOL, "name": "Composure", "short": "Comp", "min": 0, "max": CAP, "start": 0, "show": "bar", "max_stat": POOL, "start_full": true,
        "bands": [
            {"id": "shaken", "name": "Shaken", "from": 0},
            {"id": "rattled", "name": "Rattled", "from": 1, "statuses": [{"status": "weaken", "magnitude": LOW_WEAKEN}]},
            {"id": "steady", "name": "Steady", "from_pct": HALF_PCT}],
        "triggers": [{"on": "ally_downed", "when": {"range": WITHIN}, "effects": [{"type": "meter", "meter": POOL, "delta": delta}], "limit": "none"}]});
    if let Some(m) = delta_max {
        pool.as_object_mut().unwrap().insert("delta_max".into(), json!(m));
    }
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "derived_stats": [derived],
        "unit_meters": [pool],
        "units": [unit("p_hero", "party", 14), unit("p_near", "party", 13), unit("p_far", "party", 12), unit("p_ally", "party", 11), unit("p_foe", "enemy", 10)],
    }))
}

fn rules(list: &[&str], pools: &[&str]) -> Ruleset {
    let mut r = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": list}).as_object().unwrap());
    r.numbers.unit_pools = pools.iter().map(|p| p.to_string()).collect();
    r
}

fn on() -> Ruleset {
    rules(&[UNIT_METERS, TRIGGERS], &[POOL])
}

fn set_stat(u: &mut Unit, stat: &str, v: i32) {
    u.stats.core.insert(stat.to_string(), v);
}

/// The derived row's value for KNOW `know` and CONS `cons`, clamped as the row clamps it.
fn max_for(know: i32, cons: i32) -> i32 {
    (know * PER_KNOW + cons * PER_CONS).clamp(0, CAP)
}

/// Hero (4,4) and the ally (4,5) side by side, `near` 2 tiles from the ally (3,6), `far` 3 tiles from it (4,8), the
/// foe (5,5) beside the ally. Each unit's KNOW and CONS from `stats` (in unit order); the meters restart from them.
fn session(cat: Catalog, ruleset: Ruleset, stats: [(i32, i32); 5]) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_near", "p_far", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(cat), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (NEAR, V2::new(3, 6)), (FAR, V2::new(4, 8)), (ALLY, V2::new(4, 5)), (FOE, V2::new(5, 5))] {
        s.w.grid.positions[c] = Some(cell);
    }
    for (c, (know, cons)) in stats.into_iter().enumerate() {
        set_stat(&mut s.w.units[c], "KNOW", know);
        set_stat(&mut s.w.units[c], "CONS", cons);
    }
    s.w.meters = meter::Meters::default();
    meter::battle_start(&mut s.w);
    s
}

const STATS: [(i32, i32); 5] = [(7, 4), (10, 5), (12, 6), (20, 8), (60, 10)];

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn command(w: &World, c: UnitId, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

fn book(res: &mut Resolver, c: UnitId, cmd: Command, start: i32) {
    assert!(res.schedule(c, cmd, start), "booked at {start}");
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

/// The foe knocks the ally (left at 1 HP) with a strike at block 1.
fn knock_ally(w: &mut World) -> Resolver {
    w.units[ALLY].hp = 1;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, command(w, FOE, PlanStep::engage(sid(w, "strike"), ALLY), "strike"), 1);
    run(w, &mut res, 1, 1);
    assert!(!w.units[ALLY].is_alive(), "the ally is knocked");
    res
}

fn band(w: &World, c: UnitId) -> String {
    meter::shown(w, c).into_iter().find(|m| m.id == POOL).map(|m| m.band).unwrap_or_default()
}

/// Every unit holds the pool, full at its own formula max (the foe's 130 clamped to 100); the pool view and the screens'
/// row read the same range, and the band is the top one.
#[test]
fn every_unit_starts_full_at_its_formula_max() {
    let s = session(catalog(DOWNED, None), on(), STATS);
    for (c, (know, cons)) in STATS.into_iter().enumerate() {
        let max = max_for(know, cons);
        let view = meter::pool(&s.w, c, POOL).expect("a pool");
        assert_eq!((view.current, view.min, view.max), (max, 0, max), "unit {c}");
        let shown = meter::shown(&s.w, c).into_iter().find(|m| m.id == POOL).expect("shown");
        assert_eq!((shown.value, shown.max, shown.band.as_str()), (max, max, "Steady"), "unit {c}");
    }
    assert_eq!(max_for(STATS[FOE].0, STATS[FOE].1), CAP, "the fixture's foe is above the cap");
}

/// The off switch: without `unit_pools`, or without the rule unit_meters, no unit holds the pool, and a knock and a Rest
/// log no meter line.
#[test]
fn without_unit_pools_or_the_rule_no_unit_holds_it() {
    for ruleset in [rules(&[UNIT_METERS, TRIGGERS], &[]), rules(&[TRIGGERS], &[POOL]), Ruleset::classic()] {
        let mut s = session(catalog(DOWNED, None), ruleset, STATS);
        assert!(s.w.meters.gauges.is_empty());
        let res = knock_ally(&mut s.w);
        assert!(res.log.iter().all(|l| !l.starts_with("[Meter]")), "{:?}", res.log);
        assert!((0..5).all(|c| meter::pool(&s.w, c, POOL).is_none()));
    }
}

/// A knocked friend takes 10 from each friend within 2 tiles (the hero at 1, `near` at 2), not from `far` (3 tiles)
/// nor from the foe; the hero (max 18) drops below half (9) into the band from 1 and holds weaken; a Rest gives 5 back
/// at its start, which lifts it to half again: the band lets weaken go.
#[test]
fn a_knocked_friend_within_range_shakes_and_rest_steadies() {
    let mut s = session(catalog(DOWNED, None), on(), STATS);
    let w = &mut s.w;
    let full = |c: UnitId| max_for(STATS[c].0, STATS[c].1);
    let half = |c: UnitId| bands::pct_point(HALF_PCT as i64, full(c) as i64) as i32;
    let res = knock_ally(w);
    assert_eq!(meter::value(w, HERO, POOL), Some(full(HERO) + DOWNED));
    assert_eq!(meter::value(w, NEAR, POOL), Some(full(NEAR) + DOWNED), "2 tiles is within 2");
    assert_eq!(meter::value(w, FAR, POOL), Some(full(FAR)), "3 tiles is not");
    assert_eq!(meter::value(w, FOE, POOL), Some(full(FOE)), "not the foe's friend");
    assert!(full(HERO) + DOWNED < half(HERO), "the fixture's hero drops below half");
    assert!(full(NEAR) + DOWNED >= half(NEAR), "the fixture's near friend stays at half or above");
    assert_eq!((band(w, HERO).as_str(), band(w, NEAR).as_str()), ("Rattled", "Steady"));
    assert!(w.units[HERO].has_status("weaken") && !w.units[NEAR].has_status("weaken"));
    let line = format!("({} -> {})", full(HERO), full(HERO) + DOWNED);
    assert!(res.log.iter().any(|l| l.starts_with("[Meter] p_hero's Composure: ") && l.ends_with(&line)), "{:?}", res.log);

    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, command(w, HERO, PlanStep::self_skill(sid(w, "rest")), "rest"), 1);
    run(w, &mut res, 1, 1);
    let after = full(HERO) + DOWNED + REST_GAIN;
    assert_eq!(meter::value(w, HERO, POOL), Some(after));
    assert!(after >= half(HERO), "the fixture's Rest lifts the hero to half");
    assert_eq!(band(w, HERO), "Steady");
    assert!(!w.units[HERO].has_status("weaken"), "the band let weaken go");
}

/// At 0 the pool is in the band from 0; `delta_max` bounds one change: the default 10 cuts a -25 to -10, a row's 30
/// lets the -25 through (and the value stops at 0).
#[test]
fn delta_max_bounds_a_change_and_zero_is_the_bottom_band() {
    for (delta_max, want) in [(None, -rpgkit_data::METER_DELTA_MAX), (Some(30), -25)] {
        let mut s = session(catalog(-25, delta_max), on(), STATS);
        let w = &mut s.w;
        let full = |c: UnitId| max_for(STATS[c].0, STATS[c].1);
        knock_ally(w);
        assert_eq!(meter::value(w, NEAR, POOL), Some(full(NEAR) + want), "delta_max {delta_max:?}");
        if delta_max.is_some() {
            assert!(full(HERO) < 25, "the fixture's hero empties");
            assert_eq!((meter::value(w, HERO, POOL), band(w, HERO).as_str()), (Some(0), "Shaken"));
        }
    }
}

/// A full fight with the pool on runs to its end (both sides act by the planners) and every unit still holds it in
/// range.
#[test]
fn a_fight_with_the_pool_on_runs() {
    let mut s = session(catalog(DOWNED, None), on(), STATS);
    for _ in 0..4 {
        if !s.plan_confirmed {
            s.auto_draft("balanced");
            assert!(s.confirm_plan());
        }
        s.resolve_round();
        if !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    assert!(s.current_round > 1 || s.combat_over, "the fight ran");
    for c in 0..5 {
        let v = meter::pool(&s.w, c, POOL).expect("a pool");
        assert!((0..=v.max).contains(&v.current), "unit {c}: {v:?}");
    }
}
