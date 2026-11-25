//! PR6-7 (`docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` sections 1-2, the battle side) on a scratch catalog built here (no
//! shipped data). Expectations come from the rules: a `start_meter` grant adds its value to every holder of its meter
//! after the holders' own start (within floor..max), only to units whose track meets its `requires_track`, and no grant
//! leaves the start as it was; an arena hazard's `every_rounds` N adds rounds N, 2N, ... to its `rounds`; a `when_meter`
//! gate (arena effect, ground row) spares a unit below it or holding no such meter; a ground row whose only outcome is a
//! meter change records and logs a hit.

use rpgkit_battle::arena::{self, Hazard};
use rpgkit_battle::grid::V2;
use rpgkit_battle::ground::{self, Event, Placed};
use rpgkit_battle::meter;
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::{Catalog, HazardDef, PassiveDef, StartMeter, UnitDef, EVERY_ROUNDS_HORIZON};
use rpgkit_rules::ruleset::{Ruleset, ARENA_HAZARDS, GROUND_HAZARDS, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
/// The strain-like meter: range MIN..MAX, start START.
const MIN: i32 = 0;
const MAX: i32 = 100;
const START: i32 = 30;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let meters = vec![json!({"id": "tension", "name": "Tension", "short": "Te", "min": MIN, "max": MAX, "start": START, "show": "bar"})];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "power": 100, "armor": 0, "speed": 10, "attack_range": 1, "skill_cap": 5});
    let mut cat = Catalog::from_value(&json!({"skills": skills, "unit_meters": meters, "units": [unit("p_hero", "party"), unit("p_ally", "party"), unit("p_foe", "enemy")]}));
    cat.passives.insert("trait0".into(), PassiveDef { id: "trait0".into(), name: "Trait".into(), effects: vec![json!({"type": "meter", "meter": "tension"})], slot: String::new(), icon: String::new() });
    cat
}

/// The hero and the ally hold the tension meter (passive `trait0`); the foe holds none.
struct Holders;

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if r == "p_foe" { Vec::new() } else { vec!["trait0".to_string()] };
        Overrides { passives, ..Overrides::default() }
    }
}

fn ruleset(extra: &[&str]) -> Ruleset {
    let rules: Vec<&str> = [TRIGGERS, UNIT_METERS].into_iter().chain(extra.iter().copied()).collect();
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

fn grant(o: Value) -> StartMeter {
    StartMeter::from_obj(o.as_object().unwrap()).expect("a grant")
}

/// Hero (4,4), ally (4,5), foe (5,4); `rules` added; `extra` encounter keys; `grants` the caller's start_meter.
fn session_with(rules: &[&str], extra: Value, grants: Vec<StartMeter>) -> Session {
    let mut row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset(rules);
    def.start_meter = grants;
    let mut s = Session::start(Arc::new(catalog()), def, &Holders);
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn registered() -> Resolver {
    let mut res = Resolver::new(BLOCKS, 1);
    for c in [HERO, ALLY, FOE] {
        res.register(c);
    }
    res
}

// ---- start_meter -----------------------------------------------------------------------------------------------------

/// The entry reads only with a meter and a non-zero value; its value is held to -100..100.
#[test]
fn a_start_meter_entry_needs_a_meter_and_a_value() {
    assert!(StartMeter::from_obj(json!({"meter": "tension"}).as_object().unwrap()).is_none());
    assert!(StartMeter::from_obj(json!({"value": 5}).as_object().unwrap()).is_none());
    assert_eq!(grant(json!({"meter": "tension", "value": 500})).value, StartMeter::VALUE_MAX);
    let g = grant(json!({"meter": "tension", "value": 10, "requires_track": {"id": "rot", "at_least": 1}, "unless_flag": "x.done"}));
    assert_eq!(g.requires_track.map(|t| (t.id, t.at_least)), Some(("rot".to_string(), 1)));
    assert_eq!(g.unless_flag, "x.done");
}

/// The battle's grants add to every holder after its own start, never to a unit without the meter, within the max.
#[test]
fn a_start_meter_grant_raises_every_holder_within_the_max() {
    let s = session_with(&[], json!({}), vec![grant(json!({"meter": "tension", "value": 15}))]);
    assert_eq!(meter::value(&s.w, HERO, "tension"), Some(START + 15));
    assert_eq!(meter::value(&s.w, ALLY, "tension"), Some(START + 15));
    assert_eq!(meter::value(&s.w, FOE, "tension"), None);
    assert_eq!(meter::peak(&s.w, HERO, "tension"), Some(START + 15), "the start counts toward the peak");
    let capped = session_with(&[], json!({}), vec![grant(json!({"meter": "tension", "value": 60})), grant(json!({"meter": "tension", "value": 60}))]);
    assert_eq!(meter::value(&capped.w, HERO, "tension"), Some(MAX));
    let low = session_with(&[], json!({}), vec![grant(json!({"meter": "tension", "value": -100}))]);
    assert_eq!(meter::value(&low.w, HERO, "tension"), Some(MIN));
}

/// `requires_track` reads the unit's track value (none held = 0): only a unit at `at_least` or more gains.
#[test]
fn a_track_gate_grants_only_units_at_its_stage() {
    let mut s = session_with(&[], json!({}), Vec::new());
    s.w.meters = Default::default();
    s.w.units[HERO].tracks = Some([("rot".to_string(), 2)].into_iter().collect());
    s.w.units[ALLY].tracks = Some([("rot".to_string(), 0)].into_iter().collect());
    meter::battle_start_with(&mut s.w, &[grant(json!({"meter": "tension", "value": 10, "requires_track": {"id": "rot", "at_least": 1}}))]);
    assert_eq!(meter::value(&s.w, HERO, "tension"), Some(START + 10));
    assert_eq!(meter::value(&s.w, ALLY, "tension"), Some(START), "stage 0 is below the gate");
}

/// No grant: the start as `battle_start` sets it; without the unit meters rule nothing is held at all.
#[test]
fn no_grant_or_no_rule_changes_nothing() {
    let plain = session_with(&[], json!({}), Vec::new());
    let mut again = session_with(&[], json!({}), Vec::new());
    again.w.meters = Default::default();
    meter::battle_start(&mut again.w);
    assert_eq!(plain.w.meters, again.w.meters);
    assert_eq!(meter::value(&plain.w, HERO, "tension"), Some(START));
    let mut s = session_with(&[], json!({}), Vec::new());
    s.w.ruleset = Ruleset::classic();
    s.w.meters = Default::default();
    meter::battle_start_with(&mut s.w, &[grant(json!({"meter": "tension", "value": 10}))]);
    assert!(s.w.meters.gauges.is_empty());
}

// ---- every_rounds ----------------------------------------------------------------------------------------------------

fn hazard(o: Value) -> Hazard {
    Hazard::from_obj(o.as_object().unwrap())
}

/// `every_rounds` N adds rounds N, 2N, ... (up to the horizon) to the listed rounds, each once; without it the list
/// stays as stored (empty = every round).
#[test]
fn every_rounds_expands_into_the_rounds_list() {
    let base = json!({"id": "toll", "name": "Toll", "columns": [3], "blocks": [6], "effect": {"damage": 5}});
    let mut o = base.clone();
    o["every_rounds"] = json!(2);
    let h = hazard(o);
    let want: Vec<i32> = (1..=EVERY_ROUNDS_HORIZON / 2).map(|k| 2 * k).collect();
    assert_eq!(h.rounds, want);
    assert!(!h.due_in(1) && h.due_in(2) && !h.due_in(3) && h.due_in(4));
    let mut o = base.clone();
    o["every_rounds"] = json!(3);
    o["rounds"] = json!([1, 3]);
    let h = hazard(o);
    assert_eq!(&h.rounds[..4], &[1, 3, 6, 9]);
    assert!(hazard(base.clone()).rounds.is_empty());
    let mut o = base;
    o["every_rounds"] = json!(0);
    assert!(hazard(o).rounds.is_empty(), "0 is off");
}

// ---- when_meter ------------------------------------------------------------------------------------------------------

/// An arena effect's gate strikes only units holding the meter at `at_least` or more.
#[test]
fn an_arena_meter_gate_spares_units_below_it() {
    let row = |at_least: i32| json!({"arena_hazards": [{"id": "pulse", "name": "Pulse", "cells": [[4, 4], [4, 5], [5, 4]], "blocks": [1], "effect": {"damage": 7, "meter": {"meter": "tension", "delta": 5}, "when_meter": {"id": "tension", "at_least": at_least}}}]});
    let mut s = session_with(&[ARENA_HAZARDS], row(START), Vec::new());
    s.w.meters.gauges.iter_mut().filter(|g| g.unit == ALLY).for_each(|g| g.value = START - 1);
    let mut res = registered();
    let strikes = arena::block_end(&mut res, &mut s.w, 1, 1, &[]);
    let struck: Vec<UnitId> = strikes[0].hits.iter().map(|h| h.unit).collect();
    assert_eq!(struck, vec![HERO], "the ally is below the gate, the foe holds no meter");
    assert_eq!(meter::value(&s.w, HERO, "tension"), Some(START + 5));
    let mut s = session_with(&[ARENA_HAZARDS], row(START + 1), Vec::new());
    let mut res = registered();
    assert!(arena::block_end(&mut res, &mut s.w, 1, 1, &[])[0].hits.is_empty());
    // no gate: every unit on its cells
    let mut s = session_with(&[ARENA_HAZARDS], json!({"arena_hazards": [{"id": "pulse", "name": "Pulse", "cells": [[4, 4], [4, 5], [5, 4]], "blocks": [1], "effect": {"damage": 7}}]}), Vec::new());
    let mut res = registered();
    assert_eq!(arena::block_end(&mut res, &mut s.w, 1, 1, &[])[0].hits.len(), 3);
}

/// A ground row's gate spares units below it; a meter change alone is a recorded, logged hit.
#[test]
fn a_ground_meter_gate_spares_and_a_meter_alone_is_a_hit() {
    let mut s = session_with(&[GROUND_HAZARDS], json!({}), Vec::new());
    let def = HazardDef::from_obj(json!({"id": "hum", "name": "Hum", "on": "both", "meter": {"meter": "tension", "delta": 4}, "when_meter": {"id": "tension", "at_least": START}}).as_object().unwrap());
    assert_eq!(def.when_meter.as_ref().map(|g| g.at_least), Some(START));
    assert_eq!(HazardDef::from_obj(json!({"id": "x", "when_meter": {}}).as_object().unwrap()).when_meter, None);
    s.w.meters.gauges.iter_mut().filter(|g| g.unit == ALLY).for_each(|g| g.value = START - 1);
    for c in [V2::new(4, 4), V2::new(4, 5), V2::new(5, 4)] {
        s.w.grid.ground.insert(c, Placed { def: def.clone(), by: FOE, round: 1, last_round: 3 });
    }
    let mut res = registered();
    let mut rng = aegis_core::rng::GodotRng::from_i64(1);
    let note = ground::enter(&mut s.w, &mut rng, HERO, V2::new(4, 4), &mut Vec::new());
    assert_eq!(note.as_deref(), Some("[Hazard] p_hero enters Hum"), "the meter change alone makes the hit");
    assert_eq!(ground::enter(&mut s.w, &mut rng, ALLY, V2::new(4, 5), &mut Vec::new()), None, "below the gate");
    assert_eq!(ground::enter(&mut s.w, &mut rng, FOE, V2::new(5, 4), &mut Vec::new()), None, "no such meter");
    ground::settle(&mut res, &mut s.w);
    assert_eq!(meter::value(&s.w, HERO, "tension"), Some(START + 4));
    assert_eq!(meter::value(&s.w, ALLY, "tension"), Some(START - 1));
    ground::round_end(&mut res, &mut s.w);
    assert_eq!(meter::value(&s.w, HERO, "tension"), Some(START + 8));
    let hits: Vec<UnitId> = s.w.ground.take().into_iter().filter_map(|e| if let Event::Hit(h) = e { Some(h.unit) } else { None }).collect();
    assert_eq!(hits, vec![HERO, HERO], "on entering and at round end");
    assert!(res.log.iter().any(|l| l == "[Hazard] p_hero is caught in Hum"), "{:?}", res.log);
}
