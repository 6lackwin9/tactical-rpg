//! BATTLE-UI2 (the battle screen contract 5, the docked foe card) on a scratch catalog
//! built here (no shipped data): `intent::foe_card` reads the round as it will run (`planned_schedule`), never a list
//! of its own. Expectations come from the plan and the resolver: the card's rows are the plan's entries on the foe
//! (same skills and blocks), the pips each hit breaks are the pips the round then takes at that block, the KO block is
//! the block the round takes the foe to 0 HP (hits after it carry the after-KO tag), and the crit star follows the
//! chance over every crit outcome (a sure crit stars the block a crit KOs by, no crit chance gives no star).

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent::{self, FoeCard};
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::poise;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, POISE, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;

fn catalog(foe_hp: i32) -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "c_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "stagger": 14, "target_mode": "ENEMY"}),
        json!({"id": "c_heavy", "name": "Heavy", "kind": "ATTACK", "level": 1, "blocks": 2, "power_mult": 1.0, "stagger": 30, "target_mode": "ENEMY"}),
    ];
    let unit = |id: &str, side: &str, hp: i32, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": hp, "power": 100, "armor": 10, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": 60});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("c_hero", "party", 5000, 14), unit("c_ally", "party", 5000, 12), unit("c_foe", "enemy", foe_hp, 10)]}))
}

/// The hero west and the ally east of the foe, both next to it, under the timeline with poise pips; the foe at `foe_hp`.
fn session(foe_hp: i32) -> Session {
    let row = json!({"id": "c_fight", "name": "Fight", "party": ["c_hero", "c_ally"], "enemies": ["c_foe"], "total_blocks": BLOCKS, "unit_ap": 12, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [POISE], "poise_mode": "pips"}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog(foe_hp)), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(6, 4)), (ALLY, V2::new(8, 4)), (FOE, V2::new(7, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.resolver.scheduled.clear();
    for c in [HERO, ALLY, FOE] {
        s.resolver.register(c);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// Each unit's skills on the foe, in order.
fn plan_of(w: &World, steps: &[(UnitId, &str)]) -> Plan {
    let mut plan = Plan::new();
    for (c, skill) in steps {
        assert_eq!(plan.add_step(w, *c, PlanStep::engage(sid(w, skill), FOE), true), "", "{skill} for {}", w.units[*c].name);
    }
    plan
}

fn card(s: &Session, plan: &Plan) -> FoeCard {
    intent::foe_card(&s.w, &intent::planned_schedule(&s.w, &s.resolver.scheduled, plan), FOE)
}

/// Commits `plan` and runs the round: per block, the foe's HP and pips after it, and the log.
fn run(s: &mut Session, plan: &Plan) -> (Vec<(i32, i32, i32)>, Vec<String>) {
    plan.commit(&s.w, &mut s.resolver);
    let mut after = Vec::new();
    let mut log = Vec::new();
    for b in 1..=BLOCKS {
        for i in s.resolver.prepare_block(&mut s.w, b) {
            s.resolver.execute_single(&mut s.w, i);
            log.push(s.resolver.scheduled[i].command.log.clone());
        }
        let pips = poise::state(&s.w, FOE).map_or(0, |st| poise::pips_now(&s.w.ruleset.numbers, st));
        after.push((b, s.w.units[FOE].hp, pips));
    }
    (after, log)
}

const STEPS: [(UnitId, &str); 5] = [(HERO, "c_jab"), (HERO, "c_heavy"), (HERO, "c_jab"), (ALLY, "c_jab"), (ALLY, "c_heavy")];

/// The card's rows are the live plan's: each hero's entries on the foe with the same skill, first block and landing
/// block, in block order, and nothing else.
#[test]
fn the_rows_are_the_live_plan_entries_on_the_foe() {
    let s = session(5000);
    let plan = plan_of(&s.w, &STEPS);
    let c = card(&s, &plan);
    for u in [HERO, ALLY] {
        let entries: Vec<(String, i32, i32)> = plan.compile(&s.w, u).entries.iter().filter(|e| e.target == Some(FOE)).map(|e| (s.w.cat.skills[e.skill].id.clone(), e.start_block, e.end_block(&s.w))).collect();
        let rows: Vec<(String, i32, i32)> = c.hits.iter().filter(|h| h.unit == u).map(|h| (h.skill.clone(), h.start, h.impact)).collect();
        assert_eq!(rows, entries, "unit {u}");
    }
    assert_eq!(c.hits.len(), STEPS.len());
    assert!(c.hits.windows(2).all(|p| p[0].impact <= p[1].impact), "block order: {:?}", c.hits);
    let blocks = c.by_block();
    assert_eq!(blocks.iter().map(|b| b.1.len()).sum::<usize>(), c.hits.len());
    assert!(blocks.iter().all(|(b, hs)| hs.iter().all(|h| h.impact == *b)));
    assert_eq!(c.total(None), c.total(Some(HERO)) + c.total(Some(ALLY)));
    // a hit's damage is the one forecast convention's (`hits_on`)
    let sched = intent::planned_schedule(&s.w, &s.resolver.scheduled, &plan);
    let by_hero: Vec<i32> = intent::hits_on(&s.w, &sched, HERO, FOE).iter().map(|h| h.damage).collect();
    assert_eq!(c.hits.iter().filter(|h| h.unit == HERO).map(|h| h.damage).collect::<Vec<_>>(), by_hero);
}

/// Each hit breaks the pips the round then takes at its block (pips settle per hit, never pooled), and the block the
/// bar empties is the BREAK block, its hit marked with the split shield and none after it breaking more.
#[test]
fn the_pips_each_hit_breaks_are_what_the_round_takes() {
    let mut s = session(5000);
    let plan = plan_of(&s.w, &STEPS);
    let c = card(&s, &plan);
    let start = c.pips;
    assert!(c.threshold > 0 && c.max_pips > 0 && start > 0, "a bar of pips: {c:?}");
    let (after, log) = run(&mut s, &plan);
    assert!(!log.iter().any(|l| l.contains("CRIT")), "seed without a crit: {log:?}");
    let mut before = start;
    let mut seen_break = false;
    for (b, _, pips) in &after {
        let hits: Vec<_> = c.hits.iter().filter(|h| h.impact == *b).collect();
        let lost: i32 = hits.iter().map(|h| h.pips).sum();
        if hits.iter().any(|h| h.breaks) {
            assert_eq!(*pips, 0, "block {b}: the bar empties: {hits:?}");
        } else if !seen_break {
            assert_eq!(before - pips, lost, "block {b}: {hits:?}");
        }
        assert!(hits.iter().all(|h| h.pips <= 3), "at most the per-hit cap");
        if hits.iter().any(|h| h.breaks) {
            assert_eq!(c.break_block, Some(*b));
            seen_break = true;
        }
        before = *pips;
    }
    let broke = c.hits.iter().position(|h| h.breaks);
    if let Some(k) = broke {
        assert!(c.hits[k + 1..].iter().all(|h| h.on_broken && h.pips == 0 && !h.breaks), "{:?}", c.hits);
    }
    assert_eq!(c.break_block.is_some(), broke.is_some());
}

/// The KO block is the block the round takes the foe to 0 HP; hits after it carry the after-KO tag, the rest do not.
#[test]
fn the_ko_block_is_where_the_round_downs_the_foe() {
    let probe = card(&session(5000), &plan_of(&session(5000).w, &STEPS));
    // an HP the plan's base damage reaches before its last hit lands
    let last = probe.hits.last().unwrap().impact;
    let hp = probe.hits.iter().filter(|h| h.impact < last).map(|h| h.damage).sum::<i32>() - 1;
    assert!(hp > 0);
    let mut s = session(hp);
    let plan = plan_of(&s.w, &STEPS);
    let c = card(&s, &plan);
    let ko = c.ko_block.expect("a KO block");
    assert!(ko < last);
    assert!(c.hits.iter().all(|h| h.after_ko == (h.impact > ko)), "{:?}", c.hits);
    assert!(c.hits.iter().any(|h| h.after_ko));
    let (after, log) = run(&mut s, &plan);
    assert!(!log.iter().any(|l| l.contains("CRIT")), "seed without a crit: {log:?}");
    let down = after.iter().find(|(_, hp, _)| *hp <= 0).map(|(b, _, _)| *b);
    assert_eq!(down, Some(ko), "{after:?}");
    assert!(session(5000).w.units[FOE].hp > 0 && card(&session(5000), &plan_of(&session(5000).w, &STEPS)).ko_block.is_none());
}

/// The crit star: with a sure crit the foe is down (chance 100 %) at the first block a crit of the hits so far reaches
/// its HP, before the base KO block; with no crit chance there is none.
#[test]
fn the_crit_star_marks_the_block_crits_down_the_foe_by_half_the_time() {
    // two jabs in blocks 1 and 2 (base KO at the third), the HP one over two base jabs
    let steps = [(HERO, "c_jab"), (HERO, "c_jab"), (HERO, "c_jab")];
    let probe = card(&session(5000), &plan_of(&session(5000).w, &steps));
    let d: Vec<i32> = probe.hits.iter().map(|h| h.damage).collect();
    let hp = d[0] + d[1] + 1;
    let mut s = session(hp);
    s.w.units[HERO].passive_crit = 1000.0;
    let plan = plan_of(&s.w, &steps);
    let sure = card(&s, &plan);
    assert_eq!(sure.ko_block, Some(sure.hits[2].impact));
    assert!(sure.hits.iter().all(|h| h.crit_chance == 1.0 && h.crit_damage > h.damage));
    // the first block by which the crits of the hits so far reach the HP
    let mut sum = 0;
    let want = sure.hits.iter().find(|h| {
        sum += h.crit_damage;
        sum >= hp
    });
    assert_eq!(sure.crit_star, want.map(|h| h.impact).filter(|b| *b < sure.ko_block.unwrap()));
    assert!(sure.crit_star.is_some(), "{sure:?}");
    s.w.units[HERO].passive_crit = -1000.0;
    let none = card(&s, &plan);
    assert!(none.hits.iter().all(|h| h.crit_chance == 0.0));
    assert_eq!((none.ko_block, none.crit_star), (sure.ko_block, None));
}

/// Sets the hero's crit chance to exactly `pct` % (its passive crit moves the chance one for one).
fn crit_at(s: &mut Session, pct: f64) {
    let now = rpgkit_rules::damage::crit_chance(&s.w.units[HERO]);
    s.w.units[HERO].passive_crit += pct - now;
    assert!((rpgkit_rules::damage::crit_chance(&s.w.units[HERO]) - pct).abs() < 1e-9, "crit chance {pct}");
}

/// The 50 % boundary (contract 5, "reaches 50%"): two jabs, the base KO at the second, the first jab's crit alone
/// downs the foe; at a 50 % crit chance the foe is down by block 1 exactly half the time (a star), at 40 % it is not.
#[test]
fn the_crit_star_holds_at_exactly_half_and_not_below() {
    let steps = [(HERO, "c_jab"), (HERO, "c_jab")];
    let probe = card(&session(5000), &plan_of(&session(5000).w, &steps));
    let first = &probe.hits[0];
    let hp = first.damage + 1;
    assert!(probe.hits[1].impact > first.impact, "the jabs land in two blocks: {probe:?}");
    let mut s = session(hp);
    let plan = plan_of(&s.w, &steps);
    crit_at(&mut s, 50.0);
    let half = card(&s, &plan);
    assert_eq!(half.ko_block, Some(half.hits[1].impact), "the base KO at the second jab");
    assert!(half.hits[0].crit_damage >= hp, "the first jab's crit downs it: {half:?}");
    assert_eq!(half.hits[0].crit_chance, 0.5);
    assert_eq!(half.crit_star, Some(half.hits[0].impact));
    crit_at(&mut s, 40.0);
    let below = card(&s, &plan);
    assert_eq!((below.ko_block, below.crit_star), (half.ko_block, None));
}
