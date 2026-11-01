//! CLN-45 on a scratch catalog built here (no shipped data): the behaviours the rule `round_intent` used to switch as a
//! bundle are ruleset row fields (decision "Godot parity retired and cleanup pass 3 answers" Q-B2): `rest_ap`, `chase`,
//! `root_stops_chase`, `switch_target_step` and `stances`. Each field set on a scratch ruleset flips its behaviour alone,
//! both from classic (no rules) and from a timeline row (`round_intent`); left empty, each reads as today (the round
//! intent rule's choice with `round_intent`, the legacy one without). Expectations come from the rules: Rest pays
//! `recover_ap` on landing or `rest_gain` at round end (RULESET1_INTENT RS2), the chase (AS1, INPUT1 MR16) after a move
//! to a chosen tile and for Support, root stopping it, Switch Target before an attack on another unit than the lock
//! (TG3) and the classic stance change (IN3).

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanEntry, PlanStep};
use rpgkit_battle::round_intent::{self, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, CLASSIC, FLANK, ROUND_INTENT, TIMELINE};
use rpgkit_rules::{status, Stance};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
/// The Rest row's `recover_ap`.
const RECOVER: i32 = 2;

/// Engine skills and the test skills: a cut, a heavy, a guard and a Support haste.
fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": RECOVER}),
        json!({"id": "t_cut", "name": "Cut", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0}),
        json!({"id": "t_heavy", "name": "Heavy", "kind": "ATTACK", "level": 3, "blocks": 2, "power_mult": 2.0}),
        json!({"id": "t_fade", "name": "Fade", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.2}),
        json!({"id": "t_haste", "name": "Quicken", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY_OR_SELF", "grant_status": "haste", "status_rounds": 3}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_ally", "party"), unit("t_foe", "enemy"), unit("t_foe_b", "enemy")]}))
}

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

/// A scratch ruleset row: classic (no rules) or a timeline row (`round_intent`, `flank`), with `fields` set on it.
fn ruleset(timeline: bool, fields: Value) -> Ruleset {
    let mut row = if timeline { json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, FLANK]}) } else { json!({"id": CLASSIC, "name": "Classic", "rules": []}) };
    for (k, v) in fields.as_object().unwrap() {
        row[k] = v.clone();
    }
    Ruleset::from_row(row.as_object().unwrap())
}

/// Hero, ally, foe, foe_b (unit ids 0..4), deployed, then moved off to the far corners (tests place them).
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["t_hero", "t_ally"], "enemies": ["t_foe", "t_foe_b"], "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(0, V2::new(0, 0)), (1, V2::new(0, 8)), (2, V2::new(11, 0)), (3, V2::new(11, 8))] {
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

/// Confirms the plan and resolves the round (the enemies' plans dropped).
fn resolve(s: &mut Session) {
    assert!(s.confirm_plan());
    drop_enemy_plans(s);
    while s.phase != Phase::Replay {
        s.step_next();
    }
}

/// A heavy on the foe then three Rest blocks: (AP at the start, the heavy's AP, AP after the round, AP at the next
/// round's start).
fn rest_round(r: Ruleset) -> (i32, i32, i32, i32) {
    let mut s = session(r);
    place(&mut s.w, HERO, V2::new(4, 4));
    place(&mut s.w, FOE, V2::new(5, 4));
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_heavy"), FOE), false), "");
    for _ in 0..3 {
        assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::self_skill(sid(&s.w, "rest")), false), "");
    }
    let heavy = s.plan.entries_for(&s.w, HERO)[0].total_ap();
    let start = s.w.units[HERO].ap;
    resolve(&mut s);
    let end = s.w.units[HERO].ap;
    s.advance_round();
    (start, heavy, end, s.w.units[HERO].ap)
}

/// `rest_ap`: on_use pays `recover_ap` as each Rest lands and nothing at round end; round_end pays nothing mid-round and
/// `rest_gain` of the landed blocks at the next round's start. Empty: on_use in classic, round_end under round_intent.
#[test]
fn rest_ap_pays_on_use_or_at_round_end() {
    for timeline in [false, true] {
        for set in ["", "on_use", "round_end"] {
            let r = ruleset(timeline, json!({"rest_ap": set}));
            let round_end = set == "round_end" || (set.is_empty() && timeline);
            assert_eq!(r.rest_at_round_end(), round_end, "timeline {timeline}, rest_ap '{set}'");
            let gain = round_intent::rest_gain(&r.numbers, 3);
            let (start, heavy, end, next) = rest_round(r);
            if round_end {
                assert_eq!((end, next), (start - heavy, UNIT_AP.min(start - heavy + gain)), "timeline {timeline}, '{set}': paid at round end");
            } else {
                let landed = (0..3).fold(start - heavy, |ap, _| UNIT_AP.min(ap + RECOVER));
                assert_eq!((end, next), (landed, landed), "timeline {timeline}, '{set}': paid on landing (ap_regen 0)");
            }
        }
    }
}

fn entry(w: &World, plan: &Plan, c: UnitId, skill: &str) -> PlanEntry {
    let k = sid(w, skill);
    plan.entries_for(w, c).into_iter().find(|e| e.skill == k).unwrap_or_else(|| panic!("{skill} planned"))
}

/// `chase`: classic chases with an attack on a unit unless it follows a move to a chosen tile, and never with Support;
/// every_skill chases with both. Empty: classic in classic, every_skill under round_intent (no intent picked here, so
/// nothing is filtered).
#[test]
fn chase_classic_or_every_skill() {
    for timeline in [false, true] {
        for set in ["", "classic", "every_skill"] {
            let every = set == "every_skill" || (set.is_empty() && timeline);
            let mut s = session(ruleset(timeline, json!({"chase": set})));
            let w = &mut s.w;
            place(w, HERO, V2::new(3, 4));
            place(w, FOE, V2::new(5, 4));
            place(w, ALLY, V2::new(3, 5));
            let mut plan = Plan::new();
            assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, "t_cut"), FOE), true), "");
            let after_approach = entry(w, &plan, HERO, "t_cut").chase;
            let mut plan = Plan::new();
            assert_eq!(plan.add_step(w, HERO, PlanStep::mv(sid(w, "move"), V2::new(1, 0), 1), true), "");
            assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, "t_cut"), FOE), true), "");
            let after_chosen_tile = entry(w, &plan, HERO, "t_cut").chase;
            let mut plan = Plan::new();
            assert_eq!(plan.add_step(w, HERO, PlanStep::ally(sid(w, "t_haste"), ALLY), true), "");
            let support = entry(w, &plan, HERO, "t_haste").chase;
            assert_eq!((after_approach, after_chosen_tile, support), (true, every, every), "timeline {timeline}, chase '{set}'");
        }
    }
}

/// `root_stops_chase`: the hero cuts the foe, which steps 1 tile away before the cut lands; a rooted hero follows it
/// (and the cut lands) with off, stays (and misses it) with on. Empty: off in classic, on under round_intent.
#[test]
fn root_stops_a_chase_when_set() {
    for timeline in [false, true] {
        for set in ["", "off", "on"] {
            let stops = set == "on" || (set.is_empty() && timeline);
            let mut s = session(ruleset(timeline, json!({"root_stops_chase": set})));
            place(&mut s.w, HERO, V2::new(3, 4));
            place(&mut s.w, FOE, V2::new(4, 4));
            if timeline {
                s.set_round_intent(HERO, "assault").unwrap();
                s.set_round_target(HERO, RoundTarget::Unit(FOE)).unwrap();
            }
            assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_cut"), FOE), false), "");
            assert!(s.plan.entries_for(&s.w, HERO).iter().all(|e| skill::kind(&s.w.cat.skills[e.skill]) != Kind::Move), "the cut needs no move");
            let hp = s.w.units[FOE].hp;
            assert!(s.confirm_plan());
            drop_enemy_plans(&mut s);
            place(&mut s.w, FOE, V2::new(5, 4));
            status::grant_now(&mut s.w.units[HERO], "root", 2, 0, "test");
            while s.phase != Phase::Replay {
                s.step_next();
            }
            let (at, hit) = (s.w.pos(HERO), s.w.units[FOE].hp < hp);
            if stops {
                assert_eq!((at, hit), (V2::new(3, 4), false), "timeline {timeline}, '{set}': root stops the chase");
            } else {
                assert_eq!((at, hit), (V2::new(4, 4), true), "timeline {timeline}, '{set}': the rooted hero chases and lands");
            }
        }
    }
}

/// `switch_target_step`: on inserts Switch Target before an attack on another unit than the locked one; off attacks it
/// at once (the attack takes the lock). Empty: on in classic, off under round_intent.
#[test]
fn switch_target_step_when_set() {
    for timeline in [false, true] {
        for set in ["", "on", "off"] {
            let step = set == "on" || (set.is_empty() && !timeline);
            let mut s = session(ruleset(timeline, json!({"switch_target_step": set})));
            let w = &mut s.w;
            place(w, HERO, V2::new(4, 4));
            place(w, FOE, V2::new(5, 4));
            place(w, FOE_B, V2::new(4, 5));
            let mut plan = Plan::new();
            assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, "t_cut"), FOE), true), "");
            assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, "t_cut"), FOE_B), true), "");
            let kinds: Vec<Kind> = plan.entries_for(w, HERO).iter().map(|e| skill::kind(&w.cat.skills[e.skill])).collect();
            let want = if step { vec![Kind::Attack, Kind::Target, Kind::Attack] } else { vec![Kind::Attack, Kind::Attack] };
            assert_eq!(kinds, want, "timeline {timeline}, switch_target_step '{set}'");
        }
    }
}

/// `stances`: on runs the classic stances (a Guard skill ends in its stance and a switch costs its AP); off keeps the
/// stance and costs nothing. Empty: on in classic, off under round_intent.
#[test]
fn stances_when_set() {
    for timeline in [false, true] {
        for set in ["", "on", "off"] {
            let on = set == "on" || (set.is_empty() && !timeline);
            let r = ruleset(timeline, json!({"stances": set}));
            assert_eq!(r.stances(), on, "timeline {timeline}, stances '{set}'");
            let mut s = session(r);
            let w = &mut s.w;
            place(w, HERO, V2::new(4, 4));
            let mut plan = Plan::new();
            assert_eq!(plan.add_step(w, HERO, PlanStep::self_skill(sid(w, "t_fade")), true), "");
            let e = entry(w, &plan, HERO, "t_fade");
            let fade = &w.cat.skills[sid(w, "t_fade")];
            let want = if on { (skill::switch_cost(fade, e.stance_before), skill::stance_after(fade, e.stance_before)) } else { (0, e.stance_before) };
            assert_eq!((e.switch_ap, e.stance_after), want, "timeline {timeline}, stances '{set}'");
            if on {
                assert_ne!(e.stance_after, Stance::Neutral, "a Guard skill changes the stance");
            }
        }
    }
}
