//! HEALPCT1 (`FB-177` Option A; `docs/sessions/PK1_CORRUPTION.md` 4.3 item 5, Feed's self-heal of 4% of max HP) on a
//! scratch catalog built here (no shipped data). Expectations come from the rule: a SUPPORT skill's `heal_pct` heals that
//! percent of the target's max HP (rounded down, at least 1 HP when set, at most 100%), added to its heal formula
//! (`heal_base` + stat x `heal_mult`, then the caster's potency) and passed through `World::heal`, so a held band's
//! `heal_taken` scales it; the planner's `est_heal` (and so `ai::round_value`) reads the same amount; a skill with only
//! `heal_pct` is a heal skill (`skill::heals`, the passive group "heal"); 0 or absent changes nothing.

use rpgkit_battle::ai;
use rpgkit_battle::command::{self, Command};
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::skill;
use rpgkit_battle::world::{heal_taken, UnitId, World};
use rpgkit_data::{Catalog, PassiveDef, UnitDef};
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const HP: i32 = 500;
/// The test rows' numbers (the expectations derive from them).
const FEED_PCT: i32 = 4;
const MEND_BASE: i32 = 10;
/// The hero's tension meter starts at START, inside its band from TENSE holding `heal_taken` HEAL_TAKEN.
const TENSE: i32 = 25;
const START: i32 = 30;
const HEAL_TAKEN: i32 = -25;

/// Engine skills; Feed (self, `heal_pct` only), Mend (an ally heal: `heal_base` and `heal_pct`), Plain (an ally heal by
/// its formula only) and Full (`heal_pct` 100).
fn catalog() -> Catalog {
    let support = |id: &str, target: &str, extra: serde_json::Value| {
        let mut o = json!({"id": id, "name": id, "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": target, "reach_min": 0, "reach_max": 3});
        for (k, v) in extra.as_object().unwrap() {
            o[k] = v.clone();
        }
        o
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        support("t_feed", "SELF", json!({"heal_pct": FEED_PCT})),
        support("t_mend", "ALLY", json!({"heal_base": MEND_BASE, "heal_pct": FEED_PCT})),
        support("t_plain", "ALLY", json!({"heal_base": MEND_BASE})),
        support("t_full", "ALLY_OR_SELF", json!({"heal_pct": 100})),
    ];
    let meters = vec![json!({"id": "tension", "name": "Tension", "short": "Te", "min": 0, "max": 100, "start": START, "show": "bar",
        "bands": [{"id": "calm", "from": 0}, {"id": "tense", "from": TENSE, "effects": [{"type": "heal_taken", "value": HEAL_TAKEN}]}]})];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_meters": meters,
        "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10)],
    }))
}

/// The hero holds the tension meter (passive `trait0`) when `band` is set.
struct Holders {
    band: bool,
}

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = if self.band && r == "p_hero" { vec!["trait0".to_string()] } else { Vec::new() };
        Overrides { passives, ..Overrides::default() }
    }
}

fn session(band: bool) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [TRIGGERS, UNIT_METERS]}).as_object().unwrap());
    let mut cat = catalog();
    cat.passives.insert("trait0".into(), PassiveDef { id: "trait0".into(), name: "Trait".into(), effects: vec![json!({"type": "meter", "meter": "tension"})], slot: String::new() });
    let mut s = Session::start(Arc::new(cat), def, &Holders { band });
    for (c, cell) in [(HERO, rpgkit_battle::grid::V2::new(4, 4)), (ALLY, rpgkit_battle::grid::V2::new(4, 5)), (2, rpgkit_battle::grid::V2::new(8, 8))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn step(w: &World, id: &str, target: UnitId, c: UnitId) -> Step {
    if target == c {
        PlanStep::self_skill(sid(w, id))
    } else {
        PlanStep::ally(sid(w, id), target)
    }
}

/// `c` plans `id` on `target`: the plan and its entry's `est_heal`.
fn planned(w: &World, c: UnitId, id: &str, target: UnitId) -> (Plan, i32) {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, c, step(w, id, target, c), true), "", "{id} plans");
    let est = plan.entries_for(w, c).into_iter().find(|e| e.skill == sid(w, id)).expect("the entry").est_heal;
    (plan, est)
}

/// `c` casts `id` on `target` alone in a fresh resolver; the command after the round.
fn cast(w: &mut World, c: UnitId, id: &str, target: UnitId) -> Command {
    let (plan, _) = planned(w, c, id, target);
    let cmd = plan.entries_for(w, c).into_iter().find(|e| e.skill == sid(w, id)).expect("the entry").build_command(w, c);
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(c, cmd, 1), "booked");
    for b in 1..=BLOCKS {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
    res.scheduled.pop().unwrap().command
}

/// The percent part: the target's max HP x pct / 100, rounded down, at least 1 when set, at most the max HP; 0 without
/// `heal_pct` or on a non-SUPPORT skill.
#[test]
fn the_percent_part_rounds_down_with_at_least_one_hp() {
    let mut w = session(false).w;
    let (feed, full, plain) = (w.cat.skills[sid(&w, "t_feed")].clone(), w.cat.skills[sid(&w, "t_full")].clone(), w.cat.skills[sid(&w, "t_plain")].clone());
    assert_eq!(feed.heal_pct, FEED_PCT, "the field loads");
    assert_eq!(plain.heal_pct, 0, "absent = 0");
    assert_eq!(command::heal_pct_of(&w, HERO, &feed), HP * FEED_PCT / 100);
    assert_eq!(command::heal_pct_of(&w, HERO, &full), HP, "100% = the full max HP");
    assert_eq!(command::heal_pct_of(&w, HERO, &plain), 0);
    for max in [37, 99, 10, 1] {
        w.units[HERO].max_hp = max;
        assert_eq!(command::heal_pct_of(&w, HERO, &feed), (max * FEED_PCT / 100).max(1), "max HP {max}: rounded down, at least 1");
    }
    // it reads the target's max HP, not the caster's
    w.units[ALLY].max_hp = 2 * HP;
    assert_eq!(command::heal_pct_of(&w, ALLY, &feed), 2 * HP * FEED_PCT / 100);
    let mut attack = feed.clone();
    attack.kind = "ATTACK".into();
    assert_eq!(command::heal_pct_of(&w, HERO, &attack), 0, "only a SUPPORT skill heals");
}

/// A skill with only `heal_pct` is a heal skill (the command's heal path, the AI's heal lists, the passive group).
#[test]
fn a_percent_only_skill_is_a_heal_skill() {
    let w = session(false).w;
    for id in ["t_feed", "t_mend", "t_plain", "t_full"] {
        let s = &w.cat.skills[sid(&w, id)];
        assert!(skill::heals(s), "{id}");
        assert!(rpgkit_rules::unit::skill_in_group(s, "heal"), "{id}");
    }
    let mut none = w.cat.skills[sid(&w, "t_feed")].clone();
    none.heal_pct = 0;
    assert!(!skill::heals(&none), "no heal_pct and no formula: no heal");
}

/// The heal on a target is the formula (with the caster's potency) plus the percent of the target's max HP; a skill
/// without `heal_pct` heals exactly its formula.
#[test]
fn the_percent_adds_to_the_heal_formula() {
    let w = session(false).w;
    let (mend, plain, feed) = (sid(&w, "t_mend"), sid(&w, "t_plain"), sid(&w, "t_feed"));
    let formula = command::heal_amount(&w, HERO, plain);
    assert!(formula > 0);
    assert_eq!(command::heal_amount(&w, HERO, mend), formula, "the formula part is the same rows' formula");
    assert_eq!(command::heal_amount_on(&w, HERO, ALLY, plain), formula, "no heal_pct: the formula alone");
    assert_eq!(command::heal_amount_on(&w, HERO, ALLY, mend), formula + HP * FEED_PCT / 100);
    assert_eq!(command::heal_amount(&w, HERO, feed), 0, "Feed has no formula");
    assert_eq!(command::heal_amount_on(&w, HERO, HERO, feed), HP * FEED_PCT / 100);
}

/// Cast, the heal is the amount on the target, capped by the HP missing, and the log names it.
#[test]
fn the_command_heals_the_percent() {
    let mut w = session(false).w;
    w.units[HERO].hp = HP - 100;
    let cmd = cast(&mut w, HERO, "t_feed", HERO);
    let want = HP * FEED_PCT / 100;
    assert_eq!(w.units[HERO].hp, HP - 100 + want, "{}", cmd.log);
    assert!(cmd.log.contains(&format!("healed {want} ")), "{}", cmd.log);
    // an ally heal: the formula plus the ally's percent
    w.units[ALLY].hp = HP - 200;
    let want = command::heal_amount(&w, HERO, sid(&w, "t_mend")) + HP * FEED_PCT / 100;
    let cmd = cast(&mut w, HERO, "t_mend", ALLY);
    assert_eq!(w.units[ALLY].hp, HP - 200 + want, "{}", cmd.log);
    // capped by the HP missing
    w.units[HERO].hp = HP - 1;
    cast(&mut w, HERO, "t_feed", HERO);
    assert_eq!(w.units[HERO].hp, HP);
}

/// The percent heal runs through `World::heal`: a held band's `heal_taken` scales it after the formula (Feed under a
/// healing malus, PK1 4.3 item 5).
#[test]
fn heal_taken_scales_the_percent_heal() {
    let mut w = session(true).w;
    assert_eq!(w.units[HERO].band_heal_taken, HEAL_TAKEN, "the band holds");
    w.units[HERO].hp = HP - 100;
    let cmd = cast(&mut w, HERO, "t_feed", HERO);
    assert_eq!(w.units[HERO].hp, HP - 100 + heal_taken(HP * FEED_PCT / 100, HEAL_TAKEN), "{}", cmd.log);
}

/// The planner's estimate reads the same amount (self and ally), capped by the HP missing; the round value counts it.
#[test]
fn the_planner_estimate_and_the_round_value_read_the_percent() {
    let mut w = session(false).w;
    w.units[HERO].hp = HP - 100;
    w.units[ALLY].hp = HP - 100;
    let (plan, est) = planned(&w, HERO, "t_feed", HERO);
    assert_eq!(est, HP * FEED_PCT / 100);
    assert_eq!(ai::round_value(&w, &plan, HERO), i64::from(est), "the round value counts the estimate");
    let (_, est) = planned(&w, HERO, "t_mend", ALLY);
    assert_eq!(est, command::heal_amount_on(&w, HERO, ALLY, sid(&w, "t_mend")));
    let (_, est) = planned(&w, HERO, "t_full", ALLY);
    assert_eq!(est, 100, "capped by the HP missing");
}
