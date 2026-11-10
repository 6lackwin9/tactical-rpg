//! ROLLTABLE1 (docs/specs/ACTS23.md PK1 hook a) on a scratch catalog built here (no shipped data): the `roll_table`
//! trigger effect picks one of its rows by weight on the seeded battle RNG and runs that row's effects (their own `to`)
//! as the trigger's own. The same seed picks the same row; over many seeds the picks follow the weights; the RNG is
//! drawn only when a table fires (one draw, none for a total weight of 1); a table inside a row is skipped; a meter
//! rule's table rolls on its crossing.

use rpgkit_battle::command::Command;
use rpgkit_battle::grid::V2;
use rpgkit_battle::meter;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::trigger;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, PassiveDef, UnitDef, ROLL_TABLE};
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const FOE: UnitId = 2;
/// The rupture meter: a rise of SPIKE from START crosses its rule at MAX, which rolls a table and sets it back to 0.
const RUPTURE_MAX: i32 = 10;
const RUPTURE_START: i32 = 8;
const SPIKE: i32 = 2;
const FLAT: i32 = 10;

/// Engine skills, a jab, and Spike (rupture +SPIKE); the meter rupture whose rule at its max rolls crit 1 or crit 2
/// (equal weights) and sets the meter to 0.
fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"}),
        json!({"id": "t_spike", "name": "Spike", "kind": "FOCUS", "level": 1, "blocks": 1, "ap_cost": 1, "boost": 0.1, "boost_rounds": 1, "meters": [{"meter": "rupture", "delta": SPIKE}]}),
    ];
    let meters = vec![json!({"id": "rupture", "name": "Rupture", "short": "Ru", "min": 0, "max": RUPTURE_MAX, "start": RUPTURE_START, "show": "bar", "rules": [
        {"id": "crisis", "at": RUPTURE_MAX, "effects": [table(&[(1, crit(1)), (1, crit(2))]), {"type": "meter_set", "meter": "rupture", "value": 0}]}]})];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_meters": meters,
        "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10)],
    }))
}

/// The hero holds the rupture meter (passive `trait0`), as a world character's passive would give it.
struct Holders;

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if r == "p_hero" { vec!["trait0".to_string()] } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

fn crit(v: i32) -> Value {
    json!([{"type": "crit", "value": v}])
}

/// A `roll_table` effect of (weight, effects) rows.
fn table(rows: &[(i32, Value)]) -> Value {
    json!({"type": ROLL_TABLE, "rows": rows.iter().map(|(w, e)| json!({"weight": w, "effects": e})).collect::<Vec<_>>()})
}

/// Hero (4,4) faces the foe (5,4); the ally beside the hero at (4,5).
fn session() -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [TRIGGERS, UNIT_METERS]}).as_object().unwrap());
    let mut cat = catalog();
    cat.passives.insert("trait0".into(), PassiveDef { id: "trait0".into(), name: "Trait".into(), effects: vec![json!({"type": "meter", "meter": "rupture"})], slot: String::new() });
    let mut s = Session::start(Arc::new(cat), def, &Holders);
    for (c, cell) in [(HERO, V2::new(4, 4)), (1, V2::new(4, 5)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

/// Gives `c` a passive trigger row (source `passive:<id>`, limit none).
fn give(w: &mut World, c: UnitId, id: &str, on: &str, effects: Value) {
    let cat = w.cat.clone();
    let e = json!({"type": "trigger", "on": on, "when": {}, "effects": effects, "limit": "none"});
    w.units[c].apply_effects(&cat, &[e], &format!("passive:{id}"), false);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn command(w: &World, c: UnitId, step: Step, skill: &str) -> Command {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step, true), "", "{skill}");
    let id = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the skill's entry").build_command(w, c)
}

fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

/// The hero's crit gain from one round start under `seed` with a round_start trigger holding `effects`, the RNG draws
/// it took and the log.
fn round_start(base: &World, effects: &Value, seed: i64) -> (f64, u64, Vec<String>) {
    let mut w = base.clone();
    give(&mut w, HERO, "table", "round_start", effects.clone());
    let crit = w.units[HERO].trigger_crit;
    let mut res = Resolver::new(BLOCKS, seed);
    let draws = res.rng.draws;
    trigger::round_start(&mut res, &mut w);
    (w.units[HERO].trigger_crit - crit, res.rng.draws - draws, res.log.clone())
}

/// One seed always picks the same row (the same gain and log); across seeds both rows come up.
#[test]
fn the_same_seed_picks_the_same_row() {
    let base = session().w;
    let effects = json!([table(&[(1, crit(1)), (3, crit(2))])]);
    let mut seen = Vec::new();
    for seed in 1..=32 {
        let a = round_start(&base, &effects, seed);
        let b = round_start(&base, &effects, seed);
        assert_eq!(a, b, "seed {seed}");
        assert!(a.0 == 1.0 || a.0 == 2.0, "one row ran: {a:?}");
        assert!(a.2.iter().any(|l| l.starts_with("[Trigger] p_hero's table: p_hero gains ")), "{:?}", a.2);
        seen.push(a.0);
    }
    assert!(seen.contains(&1.0) && seen.contains(&2.0), "{seen:?}");
}

/// Over many seeds the battle picks follow the weights 1:3 (a coarse bound), and `roll_row` on one stream follows
/// 1:2:5.
#[test]
fn the_picks_follow_the_weights() {
    let base = session().w;
    let effects = json!([table(&[(1, crit(1)), (3, crit(2))])]);
    let n = 2000;
    let heavy = (1..=n).filter(|&seed| round_start(&base, &effects, seed).0 == 2.0).count();
    let share = heavy as f64 / n as f64;
    assert!((0.70..=0.80).contains(&share), "weight 3 of 4: {share}");

    let weights = [1, 2, 5];
    let t = table(&weights.iter().map(|&w| (w, crit(w))).collect::<Vec<_>>());
    let t = t.as_object().unwrap();
    let mut rng = aegis_core::rng::GodotRng::from_i64(7);
    let draws = 8000;
    let mut counts = [0usize; 3];
    for _ in 0..draws {
        counts[trigger::roll_row(&mut rng, t).expect("a row")] += 1;
    }
    let total: i32 = weights.iter().sum();
    for (i, w) in weights.iter().enumerate() {
        let (got, want) = (counts[i] as f64 / draws as f64, *w as f64 / total as f64);
        assert!((got - want).abs() < 0.03, "row {i}: {got} vs {want} ({counts:?})");
    }
    assert_eq!(trigger::row_effects(t, 2), vec![crit(5)[0].as_object().unwrap().clone()]);
    assert!(trigger::row_effects(t, 3).is_empty());
}

/// The RNG moves only when a table fires: a plain trigger draws nothing, a table of total weight 4 one draw, a single
/// row (total 1) and a table without rows none.
#[test]
fn the_rng_is_drawn_only_when_a_table_fires() {
    let base = session().w;
    assert_eq!(round_start(&base, &crit(1), 5).1, 0, "no table, no draw");
    assert_eq!(round_start(&base, &json!([table(&[(1, crit(1)), (3, crit(2))])]), 5).1, 1);
    assert_eq!(round_start(&base, &json!([table(&[(1, crit(3))])]), 5), (3.0, 0, round_start(&base, &crit(3), 5).2), "one row: no draw, same line");
    assert_eq!(round_start(&base, &json!([{"type": ROLL_TABLE, "rows": []}]), 5).1, 0);
    let mut rng = aegis_core::rng::GodotRng::from_i64(5);
    assert_eq!(trigger::roll_row(&mut rng, json!({"type": ROLL_TABLE}).as_object().unwrap()), None);
    assert_eq!(rng.draws, 0);
}

/// A picked row's effects act as the trigger's own, each with its own `to` (a foe's hit_taken table hurts the attacker
/// and raises its own crit); a table inside a row is skipped (no draw, its rows never run).
#[test]
fn a_row_runs_its_effects_with_their_own_to_and_skips_a_nested_table() {
    let mut w = session().w;
    let inner = table(&[(1, crit(50)), (1, crit(60))]);
    give(&mut w, FOE, "spite", "hit_taken", json!([table(&[(1, json!([{"type": "damage", "value": FLAT, "to": "other"}, {"type": "crit", "value": 5}, inner]))])]));
    let (hp, crit) = (w.units[HERO].hp, w.units[FOE].trigger_crit);
    let mut res = Resolver::new(BLOCKS, 1);
    let cmd = command(&w, HERO, PlanStep::engage(sid(&w, "t_jab"), FOE), "t_jab");
    assert!(res.schedule(HERO, cmd, 1));
    run(&mut w, &mut res, 1, 1);
    assert_eq!(w.units[HERO].hp, hp - FLAT, "to other: the attacker");
    assert_eq!(w.units[FOE].trigger_crit, crit + 5.0, "to self, and the nested table never ran");
    assert!(res.log.iter().any(|l| l == &format!("[Trigger] p_foe's spite: {FLAT} damage to p_hero")), "{:?}", res.log);
}

/// A meter rule's table rolls on the crossing (one of its rows runs) and the rule's `meter_set` follows; one seed
/// replays the same row.
#[test]
fn a_meter_rule_rolls_its_table_on_the_crossing() {
    let base = session().w;
    assert_eq!(meter::value(&base, HERO, "rupture"), Some(RUPTURE_START));
    let spike = |seed: i64| {
        let mut w = base.clone();
        let crit = w.units[HERO].trigger_crit;
        let mut res = Resolver::new(BLOCKS, seed);
        let cmd = command(&w, HERO, PlanStep::self_skill(sid(&w, "t_spike")), "t_spike");
        assert!(res.schedule(HERO, cmd, 1));
        run(&mut w, &mut res, 1, 1);
        let lines: Vec<String> = res.triggers.fired.iter().flat_map(|f| f.lines.clone()).filter(|l| l.starts_with("[Meter] p_hero's Rupture: ")).collect();
        (w.units[HERO].trigger_crit - crit, meter::value(&w, HERO, "rupture"), lines)
    };
    let mut seen = Vec::new();
    for seed in 1..=16 {
        let (gain, value, lines) = spike(seed);
        assert!(gain == 1.0 || gain == 2.0, "seed {seed}: one row ran ({gain})");
        assert_eq!(value, Some(0), "meter_set after the table");
        assert!(lines.iter().any(|l| l.contains("gains")), "{lines:?}");
        assert_eq!(spike(seed), (gain, value, lines), "seed {seed} replays");
        seen.push(gain);
    }
    assert!(seen.contains(&1.0) && seen.contains(&2.0), "{seen:?}");
}
