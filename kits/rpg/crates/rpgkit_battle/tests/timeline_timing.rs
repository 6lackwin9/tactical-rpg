//! RULESET1-T on a scratch catalog built here (no shipped data): enemy recovery (T1-T4: an enemy-side unit under
//! `enemy_moves` books a skill's `recovery_blocks` after its impact, a party-side unit never does, classic ignores the
//! field), the AP base `ap_cost` (T10) and enemies that pay no AP under `enemy_moves` (T11). Expectations come from
//! the rules: start `s`, blocks `b`, recovery `r` occupy `s..s+b+r-1` and land at `s+b-1`.

use indexmap::IndexMap;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, SkillDef};
use rpgkit_rules::ruleset::{Ruleset, AP_COST, ENEMY_MOVES, TIMELINE};
use rpgkit_rules::unit::BASE_AP;
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;

/// The focus skill with `b` blocks and `r` recovery blocks.
fn focus_id(b: i32, r: i32) -> String {
    format!("t_focus_b{b}_r{r}")
}

/// Engine skills, a strike, a Focus with `ap_cost` 4 at level 1, and one self-cast Focus per (blocks, recovery) pair;
/// a party hero and two enemies, fixed-stat units that own every skill.
fn catalog() -> Catalog {
    let mut skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "t_focus_ap", "name": "Priced Focus", "kind": "FOCUS", "level": 1, "blocks": 1, "ap_cost": 4, "boost": 0.1, "boost_rounds": 1}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    for b in 1..=3 {
        for r in 0..=2 {
            skills.push(json!({"id": focus_id(b, r), "name": focus_id(b, r), "kind": "FOCUS", "level": 1, "blocks": b, "recovery_blocks": r, "boost": 0.1, "boost_rounds": 1}));
        }
    }
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_foe", "enemy"), unit("t_foe_b", "enemy")]}))
}

/// A timeline ruleset with only the rules this task reads.
fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ENEMY_MOVES, AP_COST]}).as_object().unwrap())
}

fn session(ruleset: Ruleset, enemies: &[&str]) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["t_hero"], "enemies": enemies, "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 4, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    Session::start(Arc::new(catalog()), def, &IndexMap::new())
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// Plans `pad` Rest blocks and then `skill` for `c`; the planner's problem text ("" = accepted).
fn plan_after_rests(w: &World, c: UnitId, pad: i32, skill: &str) -> (Plan, String) {
    let mut plan = Plan::new();
    for _ in 0..pad {
        assert_eq!(plan.add_step(w, c, PlanStep::self_skill(sid(w, "rest")), true), "", "a Rest block");
    }
    let problem = plan.add_step(w, c, PlanStep::self_skill(sid(w, skill)), true);
    (plan, problem)
}

/// The block each scheduled action lands in, stepping the resolver's blocks on a copy of the world.
fn landing_blocks(w: &World, r: &Resolver) -> Vec<i32> {
    let (mut w, mut r) = (w.clone(), r.clone());
    let mut landed = vec![0; r.scheduled.len()];
    for block in 1..=BLOCKS {
        for i in r.prepare_block(&mut w, block) {
            landed[i] = block;
        }
    }
    landed
}

/// T1, T2, T4: an enemy under `enemy_moves` occupies s..s+b+r-1 and lands at s+b-1; a hero (and every unit in classic)
/// occupies s..s+b-1; an action whose recovery would leave the round is refused.
#[test]
fn recovery_follows_the_impact_for_enemy_side_units_only() {
    for (ruleset, timeline_on) in [(timeline(), true), (Ruleset::classic(), false)] {
        let s = session(ruleset, &["t_foe"]);
        let w = &s.w;
        let (hero, foe) = (w.allies[0], w.enemies[0]);
        for b in 1..=3 {
            for rec in 0..=2 {
                for start in 1..=3 {
                    for (c, enemy) in [(hero, false), (foe, true)] {
                        let r = if enemy && timeline_on { rec } else { 0 };
                        let (plan, problem) = plan_after_rests(w, c, start - 1, &focus_id(b, rec));
                        let fits = start + b + r - 1 <= BLOCKS;
                        let case = format!("timeline {timeline_on}, enemy {enemy}, b {b}, r {rec}, s {start}");
                        if !fits {
                            assert!(!problem.is_empty(), "{case}: the recovery leaves the round, refused (T2)");
                            continue;
                        }
                        assert_eq!(problem, "", "{case}");
                        let entry = plan.entries_for(w, c).pop().unwrap();
                        assert_eq!((entry.start_block, entry.end_block(w), entry.last_block(w), entry.recovery), (start, start + b - 1, start + b + r - 1, r), "{case}: plan entry");
                        let mut res = Resolver::new(BLOCKS, 1);
                        assert_eq!(plan.commit(w, &mut res), start as usize, "{case}: every step booked");
                        let a = res.scheduled.last().unwrap();
                        assert_eq!((a.start, a.impact, a.end), (start, start + b - 1, start + b + r - 1), "{case}: scheduled");
                        assert_eq!(*landing_blocks(w, &res).last().unwrap(), start + b - 1, "{case}: lands at its impact");
                    }
                }
            }
        }
    }
}

/// T3, T4: a unit in recovery takes no other action; its next step starts after the recovery.
#[test]
fn the_next_step_starts_after_the_recovery() {
    let s = session(timeline(), &["t_foe"]);
    let w = &s.w;
    for (c, r) in [(w.allies[0], 0), (w.enemies[0], 2)] {
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w, c, PlanStep::self_skill(sid(w, &focus_id(1, 2))), true), "");
        assert_eq!(plan.add_step(w, c, PlanStep::self_skill(sid(w, "rest")), true), "");
        let entries = plan.entries_for(w, c);
        let rest_at = 1 + 1 + r;
        assert_eq!(entries[1].start_block, rest_at, "the Rest follows the impact and {r} recovery blocks");
        let mut res = Resolver::new(BLOCKS, 1);
        assert_eq!(plan.commit(w, &mut res), 2);
        // one more 1-block action: refused in every taken block (the focus, its recovery, the Rest), booked after them
        for block in 1..=rest_at {
            assert!(!res.schedule(c, entries[1].build_command(w, c), block), "block {block} is taken");
        }
        assert!(res.schedule(c, entries[1].build_command(w, c), rest_at + 1), "the block after the Rest is free");
    }
}

/// T10: unset `ap_cost` (or the rule off, or classic) keeps the level rule; a set one is `ap_cost` + modifier in
/// 0..BASE_AP; Move and Rest keep their rules.
#[test]
fn ap_cost_replaces_the_level_only_when_set_and_on() {
    let s = session(Ruleset::classic(), &["t_foe"]);
    let mut u = s.w.units[s.w.allies[0]].clone();
    let on = timeline();
    let mut off = timeline();
    off.rules.retain(|r| *r != AP_COST);
    let row = |kind: &str, level: i32, ap: i32| SkillDef::from_obj(json!({"id": "x", "kind": kind, "level": level, "ap_cost": ap}).as_object().unwrap());
    for m in -3..=3 {
        u.ap_cost_modifier = m;
        for level in 1..=6 {
            let old = (level.clamp(1, 5) + m).clamp(1, 5);
            for kind in ["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE"] {
                assert_eq!(skill::ap_cost_by(&row(kind, level, -1), Some(&u), &on), old, "unset: the level rule");
                for ap in 0..=10 {
                    let set = row(kind, level, ap);
                    assert_eq!(skill::ap_cost_by(&set, Some(&u), &on), (ap + m).clamp(0, BASE_AP), "{kind} ap_cost {ap} mod {m}");
                    assert_eq!(skill::ap_cost_by(&set, Some(&u), &off), old, "the rule off");
                    assert_eq!(skill::ap_cost_by(&set, Some(&u), &Ruleset::classic()), old, "classic");
                }
            }
            for kind in ["MOVE", "REST"] {
                let set = row(kind, level, 7);
                assert_eq!(skill::ap_cost_by(&set, Some(&u), &on), skill::ap_cost(&set, Some(&u)), "{kind} keeps its rule");
            }
        }
    }
}

/// T10, T11 in the planner: the hero pays the row's `ap_cost` under the rule and the level rule in classic (a row
/// without `ap_cost` the level rule in both); an enemy under `enemy_moves` pays nothing, stance switch included.
#[test]
fn the_planner_charges_ap_cost_and_spares_enemies() {
    for (ruleset, timeline_on) in [(timeline(), true), (Ruleset::classic(), false)] {
        let s = session(ruleset, &["t_foe"]);
        let w = &s.w;
        for (c, enemy) in [(w.allies[0], false), (w.enemies[0], true)] {
            for id in [focus_id(2, 0), "t_focus_ap".to_string()] {
                let (plan, problem) = plan_after_rests(w, c, 0, &id);
                assert_eq!(problem, "");
                let e = plan.entries_for(w, c).pop().unwrap();
                let row = &w.cat.skills[e.skill];
                let level_rule = skill::ap_cost(row, Some(&w.units[c]));
                let want = if enemy && timeline_on {
                    0
                } else if timeline_on && row.ap_cost >= 0 {
                    row.ap_cost + w.units[c].ap_cost_modifier
                } else {
                    level_rule
                };
                let case = format!("timeline {timeline_on}, enemy {enemy}, {id}");
                assert_eq!(e.skill_ap, want, "{case}");
                if enemy && timeline_on {
                    assert_eq!(e.total_ap(), 0, "{case}: no stance switch AP either");
                }
            }
        }
    }
}

/// T11: a whole autoplayed round under `enemy_moves` leaves every enemy's AP unchanged and no AP-debt status on it;
/// in classic the same fight spends enemy AP (the check is not vacuous).
#[test]
fn enemies_pay_no_ap_under_enemy_moves() {
    for (ruleset, timeline_on) in [(timeline(), true), (Ruleset::classic(), false)] {
        let mut s = session(ruleset, &["t_foe", "t_foe_b"]);
        let mut spent = false;
        for _ in 0..4 {
            let before: Vec<i32> = s.w.enemies.iter().map(|&e| s.w.units[e].ap).collect();
            s.plan_apply_preset_all("balanced");
            assert!(s.confirm_plan(), "the plan confirms");
            while s.phase != Phase::Replay {
                s.step_next();
            }
            let after: Vec<i32> = s.w.enemies.iter().map(|&e| s.w.units[e].ap).collect();
            let acted = s.resolver.scheduled.iter().any(|a| s.w.enemies.contains(&a.unit));
            assert!(acted, "the enemies act");
            if timeline_on {
                assert_eq!(after, before, "no AP paid or gained");
                assert!(s.resolver.scheduled.iter().filter(|a| s.w.enemies.contains(&a.unit)).all(|a| a.command.ap_cost == 0 && a.command.stance_switch_cost == 0));
            } else {
                spent |= after.iter().zip(&before).any(|(a, b)| a < b);
            }
            if s.combat_over || !s.can_advance_round() {
                break;
            }
            s.advance_round();
            for &e in &s.w.enemies {
                let u = &s.w.units[e];
                if timeline_on {
                    assert!(!u.has_status("stagger") && !u.has_status("stun"), "no AP-debt status");
                    assert_eq!(u.ap, UNIT_AP, "AP stays full");
                }
            }
        }
        if !timeline_on {
            assert!(spent, "classic enemies spend AP");
        }
    }
}
