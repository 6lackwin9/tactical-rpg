//! HOOKS1-E on a scratch catalog built here (no shipped data): events and triggers under the rule `triggers` (HOOKS1
//! 8.1-8.4). Expectations come from the rules: each event fires its holder's trigger once; `limit` round fires once a
//! round, none every time, battle once a battle; effects raise no new event (8.1). `ap` is party side only and capped at
//! max AP; `grant` until guard_end ends with the guard; `reveal` shows the next round (8.2). A `guard` effect is a held
//! guard: it raises the guard, counts as guarding for P6 and `guarded_hit`, and ends with the round (8.3). A redirect
//! computes the hit against the holder with the same crit and factors: the holder takes its `pct` part, the friend the
//! rest of its own; 100 moves the whole hit; the holder never drops below 1 HP; a running skill's trigger claims the hit
//! before a passive's (8.4). With the rule off (and in classic) nothing fires, logs or draws.

use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::trigger;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::ruleset::{Ruleset, FORCED_MOVE, POISE, ROUND_INTENT, TIMELINE, TRIGGERS};
use rpgkit_rules::status;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const BAR: i32 = 40;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

/// Engine skills and the test skills: a jab (x0.5), a staggering jab (stagger 5), a smash (stagger 60: breaks a bar of
/// 40), a Guard that intercepts for an adjacent friend while it runs (redirect 100), a self heal and a poison cleanse.
fn catalog() -> Catalog {
    let attack = |id: &str, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"});
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
        attack("t_jab", json!({})),
        attack("t_jab_st", json!({"stagger": 5})),
        attack("t_smash", json!({"stagger": 60})),
        json!({"id": "t_hook", "name": "Hook Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3,
            "triggers": [{"on": "ally_hit", "when": {"range": 1}, "effects": [{"type": "redirect", "pct": 100}]}]}),
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "heal_base": 20, "target_mode": "ALLY"}),
        json!({"id": "t_purge", "name": "Purge", "kind": "SUPPORT", "level": 1, "blocks": 1, "cleanse_statuses": ["poison"], "target_mode": "ALLY"}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": BAR});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10), unit("p_foe_b", "enemy", 8)],
    }))
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

fn rules() -> Ruleset {
    timeline(&[TRIGGERS, POISE])
}

/// Hero (4,4) faces foe (5,4); the ally beside the hero at (4,5); foe_b far away.
fn session(ruleset: Ruleset) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe_b"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(5, 4)), (FOE_B, V2::new(11, 0))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s
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

fn hit(w: &World, c: UnitId, t: UnitId, skill: &str) -> Command {
    command(w, c, PlanStep::engage(sid(w, skill), t), skill)
}

fn own(w: &World, c: UnitId, skill: &str) -> Command {
    command(w, c, PlanStep::self_skill(sid(w, skill)), skill)
}

/// A support skill of `c` on itself (built directly: the planner's support rules are not under test).
fn support(w: &World, c: UnitId, skill: &str) -> Command {
    let id = sid(w, skill);
    let mut cmd = Command::new(c, 0, 1, CmdKind::Support { target: c, skill: id, last_heal: 0, last_status: String::new(), last_cleansed: Vec::new() });
    cmd.name = skill.into();
    cmd.family = skill.into();
    cmd
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

/// Gives `c` a passive trigger row (source `passive:<id>`).
fn give(w: &mut World, c: UnitId, id: &str, on: &str, when: Value, effects: Value, limit: &str) {
    let cat = w.cat.clone();
    let e = json!({"type": "trigger", "on": on, "when": when, "effects": effects, "limit": limit});
    w.units[c].apply_effects(&cat, &[e], &format!("passive:{id}"), false);
}

/// A trigger that only counts (its effect always logs a line).
fn count(w: &mut World, c: UnitId, on: &str, limit: &str) {
    give(w, c, &format!("count_{on}_{limit}"), on, json!({}), json!([{"type": "crit", "value": 1}]), limit);
}

/// How often `holder` fired a trigger on `on` (the records the session has not logged yet).
fn fired(res: &Resolver, holder: UnitId, on: &str) -> usize {
    res.triggers.fired.iter().filter(|f| f.holder == holder && f.on == on).count()
}

/// The skill multiplier and crit of the action `a` (an attack), to recompute its damage.
fn mult_and_crit(res: &Resolver, a: usize) -> (f64, bool) {
    match &res.scheduled[a].command.kind {
        CmdKind::Attack { skill_mult, last_crit, .. } => (*skill_mult, *last_crit),
        _ => panic!("an attack"),
    }
}

/// `damage::compute` of `a` on `t` now (melee, no cover; the crit as rolled).
fn expected(w: &World, a: UnitId, t: UnitId, mult: f64, crit: bool) -> i32 {
    damage::compute(&w.units[a], &w.units[t], HitInput { skill_mult: mult, force_crit: crit, ..HitInput::default() }).final_dmg
}

/// 8.1: a hit raises `hit_taken` for the target and `hit_dealt` for the attacker, once each; `guarded_hit` only while the
/// target guards (a running Defend), and then `hit_taken` too.
#[test]
fn hit_events_fire_once_each() {
    let s = session(rules());
    let mut w = s.w.clone();
    count(&mut w, HERO, "hit_dealt", "round");
    count(&mut w, FOE, "hit_taken", "round");
    count(&mut w, FOE, "guarded_hit", "round");
    let base = w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!((fired(&res, FOE, "hit_taken"), fired(&res, FOE, "guarded_hit"), fired(&res, HERO, "hit_dealt")), (1, 0, 1));
    assert!(res.log.iter().any(|l| l.starts_with("[Trigger] p_foe's count_hit_taken_round: ")), "{:?}", res.log);

    let mut w = base;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, own(&w, FOE, "guard"), 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 2);
    run(&mut w, &mut res, 1, 2);
    assert_eq!((fired(&res, FOE, "hit_taken"), fired(&res, FOE, "guarded_hit")), (1, 1));
}

/// 8.1: a hit that downs a foe raises `kill` for the attacker and `ally_downed` for the foe's friends within `range`.
#[test]
fn kills_and_downed_friends() {
    let s = session(rules());
    for (at, near) in [(V2::new(6, 4), 1), (V2::new(11, 0), 0)] {
        let mut w = s.w.clone();
        w.grid.positions[FOE_B] = Some(at);
        w.units[FOE].hp = 1;
        count(&mut w, HERO, "kill", "round");
        give(&mut w, FOE_B, "watch", "ally_downed", json!({"range": 2}), json!([{"type": "crit", "value": 1}]), "round");
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
        run(&mut w, &mut res, 1, 1);
        assert!(!w.units[FOE].is_alive());
        assert_eq!((fired(&res, HERO, "kill"), fired(&res, FOE_B, "ally_downed")), (1, near), "foe_b at {at}");
    }
}

/// 8.1: a break raises `broken` for the broken unit and `break_dealt` for the breaker (rule poise).
#[test]
fn breaks_raise_broken_and_break_dealt() {
    let s = session(rules());
    let mut w = s.w.clone();
    count(&mut w, FOE, "broken", "round");
    count(&mut w, HERO, "break_dealt", "round");
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, hit(&w, HERO, FOE, "t_smash"), 1);
    run(&mut w, &mut res, 1, 1);
    assert!(res.scheduled[a].command.staggers[0].broke.is_some(), "stagger 60 breaks a bar of {BAR}");
    assert_eq!((fired(&res, FOE, "broken"), fired(&res, HERO, "break_dealt")), (1, 1));
}

/// 8.1 `evaded`: a locked attack on a stealthed unit that moved finds nothing (GD5, rule round_intent).
#[test]
fn a_lost_attack_raises_evaded() {
    let s = session(timeline(&[TRIGGERS, ROUND_INTENT]));
    let mut w = s.w.clone();
    count(&mut w, FOE, "evaded", "round");
    status::grant_now(&mut w.units[FOE], "stealth", 2, 0, "test");
    round_intent::note_moved(&mut w, FOE);
    let mut cmd = Command::new(HERO, 0, 1, CmdKind::Attack { target: Some(FOE), skill_mult: 0.5, use_offhand: false, follow_tiles: 0, dash_tiles: 0, status_rounds: 0, min_range: 0, max_range: 0, last_damage: 0, last_crit: false, last_target: None });
    cmd.name = "Jab".into();
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, HERO, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(res.scheduled[a].command.evaded, Some(FOE));
    assert_eq!(fired(&res, FOE, "evaded"), 1);
}

/// ATTACKLOCK1 (decision "Attack lock answers" 2): a dodge raises `evaded` on the unit that dodged; `attack_lock` none
/// raises nothing for the same move.
#[test]
fn a_dodge_raises_evaded() {
    for (lock, fires) in [(rpgkit_rules::ruleset::AttackLock::Melee, 1), (rpgkit_rules::ruleset::AttackLock::None, 0)] {
        let s = session(timeline(&[TRIGGERS, ROUND_INTENT]));
        let mut w = s.w.clone();
        w.ruleset.numbers.attack_lock = lock;
        count(&mut w, FOE, "evaded", "round");
        let mut cmd = Command::new(HERO, 0, 2, CmdKind::Attack { target: Some(FOE), skill_mult: 0.5, use_offhand: false, follow_tiles: 0, dash_tiles: 0, status_rounds: 0, min_range: 0, max_range: 0, last_damage: 0, last_crit: false, last_target: None });
        cmd.name = "Jab".into();
        let mut res = Resolver::new(BLOCKS, 1);
        let a = book(&mut res, HERO, cmd, 1);
        run(&mut w, &mut res, 1, 1);
        w.grid.positions[FOE] = Some(V2::new(4, 3));
        run(&mut w, &mut res, 2, 2);
        assert_eq!(fired(&res, FOE, "evaded"), fires, "{lock:?}: {:?}", res.log);
        assert_eq!(res.scheduled[a].command.dodged.is_some(), fires == 1);
    }
}

/// 8.1: a heal that landed raises `heal_given`, a cleanse that removed a status `cleanse_given`.
#[test]
fn heals_and_cleanses_given() {
    let s = session(rules());
    let mut w = s.w.clone();
    w.units[HERO].hp -= 100;
    status::grant_now(&mut w.units[HERO], "poison", 2, 5, "test");
    count(&mut w, HERO, "heal_given", "none");
    count(&mut w, HERO, "cleanse_given", "none");
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, support(&w, HERO, "t_mend"), 1);
    book(&mut res, HERO, support(&w, HERO, "t_purge"), 2);
    run(&mut w, &mut res, 1, 2);
    assert!(!w.units[HERO].has_status("poison"));
    assert_eq!((fired(&res, HERO, "heal_given"), fired(&res, HERO, "cleanse_given")), (1, 1));
}

/// 8.1 `hp_below`: the share crossing from above `pct` to at or below fires; staying below does not; with `limit` battle
/// a second crossing in a later round fires nothing, with round it fires again.
#[test]
fn hp_below_fires_on_a_crossing() {
    let s = session(rules());
    for (limit, total) in [("battle", 1), ("round", 2)] {
        let mut w = s.w.clone();
        give(&mut w, FOE, "low", "hp_below", json!({"pct": 50}), json!([{"type": "crit", "value": 1}]), limit);
        let half = w.units[FOE].max_hp / 2;
        w.units[FOE].hp = half + 1;
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
        book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 2);
        run(&mut w, &mut res, 1, 2);
        assert!(w.units[FOE].hp < half);
        assert_eq!(fired(&res, FOE, "hp_below"), 1, "{limit}: once, staying below fires nothing");
        res.finish_round(&mut w);
        trigger::round_start(&mut res, &mut w);
        w.units[FOE].hp = half + 1;
        book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(fired(&res, FOE, "hp_below"), total, "{limit}: the second crossing");
    }
}

/// 8.1 `limit`: two hits in a round fire a round-limited trigger once and an unlimited one twice; the next round's
/// count starts again. An effect raises no new event: the foe's `damage` on the attacker fires no `hit_taken` of it.
#[test]
fn limits_and_depth_one() {
    let s = session(rules());
    let mut w = s.w.clone();
    count(&mut w, FOE, "hit_taken", "round");
    count(&mut w, FOE, "hit_taken", "none");
    give(&mut w, FOE, "spite", "hit_taken", json!({}), json!([{"type": "damage", "value": 10, "to": "other"}]), "none");
    count(&mut w, HERO, "hit_taken", "none");
    let hp = w.units[HERO].hp;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 2);
    run(&mut w, &mut res, 1, 2);
    let by_key = |res: &Resolver, src: &str| res.triggers.fired.iter().filter(|f| f.holder == FOE && f.source == src).count();
    assert_eq!((by_key(&res, "count_hit_taken_round"), by_key(&res, "count_hit_taken_none")), (1, 2));
    assert_eq!((w.units[HERO].hp, fired(&res, HERO, "hit_taken")), (hp - 20, 0), "flat damage through guard 0, no event");
    res.finish_round(&mut w);
    trigger::round_start(&mut res, &mut w);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(by_key(&res, "count_hit_taken_round"), 2, "a new round");
}

/// The foe's jab on the hero at block 1 with the ally's passive `ally_hit` redirect `pct` (the ally at `ally_hp`): the
/// world, the resolver and the action.
fn redirected(pct: i32, ally_hp: Option<i32>) -> (World, World, Resolver, usize) {
    let s = session(rules());
    let mut w = s.w.clone();
    give(&mut w, ALLY, "penance", "ally_hit", json!({"range": 1}), json!([{"type": "redirect", "pct": pct}]), "round");
    if let Some(hp) = ally_hp {
        w.units[ALLY].hp = hp;
    }
    let before = w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, FOE, hit(&w, FOE, HERO, "t_jab"), 1);
    run(&mut w, &mut res, 1, 1);
    (before, w, res, a)
}

/// 8.4: redirect 35: the ally takes 35% of the hit computed on it, the hero 65% of its own (the same crit); 100: the
/// whole hit lands on the ally (its armour and guard), the hero takes nothing; a holder at 5 HP stops at 1 and the rest
/// goes back to the hero.
#[test]
fn redirect_and_intercept() {
    let (b, w, res, a) = redirected(35, None);
    let (mult, crit) = mult_and_crit(&res, a);
    let (own, theirs) = (expected(&w, FOE, HERO, mult, crit), expected(&w, FOE, ALLY, mult, crit));
    assert_eq!(b.units[ALLY].hp - w.units[ALLY].hp, trigger::part(theirs, 35), "the ally's 35%");
    assert_eq!(b.units[HERO].hp - w.units[HERO].hp, trigger::part(own, 65), "the hero's 65%");
    assert_eq!(fired(&res, ALLY, "ally_hit"), 1);
    assert!(res.log.iter().any(|l| l.contains("p_ally takes 35% of the hit meant for p_hero")), "{:?}", res.log);
    assert_eq!(res.scheduled[a].command.hits, vec![(HERO, false)], "stagger and the rest stay on the hero");

    let (b, w, res, a) = redirected(100, None);
    assert_eq!(b.units[HERO].hp, w.units[HERO].hp, "the hero takes nothing");
    assert_eq!(b.units[ALLY].hp - w.units[ALLY].hp, expected(&w, FOE, ALLY, mult, crit));
    assert_eq!(res.scheduled[a].command.hits, vec![(ALLY, false)], "the whole hit lands on the ally");
    assert!(matches!(res.scheduled[a].command.kind, CmdKind::Attack { last_target: Some(ALLY), .. }));

    let (b, w, res, a) = redirected(100, Some(5));
    let (mult, crit) = mult_and_crit(&res, a);
    let theirs = expected(&w, FOE, ALLY, mult, crit);
    assert!(theirs > 4, "the hit is larger than the ally's HP");
    assert_eq!(w.units[ALLY].hp, 1, "a redirect never takes the holder below 1");
    assert_eq!(b.units[HERO].hp - w.units[HERO].hp, theirs - 4, "the rest goes back to the hero");
}

/// 8.4 (CQ18 a): the ally holds a passive redirect 35 and runs a Guard whose own trigger redirects 100: the Guard claims
/// the hit (the passive does not fire); without the Guard the passive redirects 35.
#[test]
fn a_running_skill_redirects_first() {
    let s = session(rules());
    let mut w = s.w.clone();
    give(&mut w, ALLY, "penance", "ally_hit", json!({"range": 1}), json!([{"type": "redirect", "pct": 35}]), "round");
    let base = w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, ALLY, own(&w, ALLY, "t_hook"), 1);
    let a = book(&mut res, FOE, hit(&w, FOE, HERO, "t_jab"), 2);
    let hero = w.units[HERO].hp;
    run(&mut w, &mut res, 1, 2);
    assert_eq!(w.units[HERO].hp, hero, "the Guard took the whole hit");
    assert_eq!(res.scheduled[a].command.hits, vec![(ALLY, false)]);
    let sources: Vec<&str> = res.triggers.fired.iter().filter(|f| f.holder == ALLY).map(|f| f.source.as_str()).collect();
    assert_eq!(sources, vec!["Hook Guard"], "the skill fired (its name), the passive did not");

    let mut w = base;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "t_jab"), 2);
    run(&mut w, &mut res, 1, 2);
    let sources: Vec<&str> = res.triggers.fired.iter().filter(|f| f.holder == ALLY).map(|f| f.source.as_str()).collect();
    assert_eq!(sources, vec!["penance"]);
    assert!(res.log.iter().any(|l| l.contains("p_ally takes 35% of the hit")));
}

/// 8.3: a `guard` 0.5 for the rest of the round set by the hero's first hit in block 3 holds at the second hit in block
/// 5: guard 0.5 on the damage, P6's guard factor, `guarded_hit` fires; it is gone after the round's end.
#[test]
fn a_held_guard_lasts_the_round() {
    let s = session(rules());
    let mut w = s.w.clone();
    give(&mut w, HERO, "brace", "hit_taken", json!({}), json!([{"type": "guard", "mitigation": 0.5, "blocks": 0}]), "round");
    count(&mut w, HERO, "guarded_hit", "round");
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "t_jab_st"), 3);
    let a = book(&mut res, FOE, hit(&w, FOE, HERO, "t_jab_st"), 5);
    run(&mut w, &mut res, 1, 4);
    assert_eq!(w.units[HERO].held_guard.map(|g| (g.mitigation, g.last_block)), Some((0.5, BLOCKS)));
    assert_eq!(fired(&res, HERO, "guarded_hit"), 0, "the first hit was not guarded");
    let hp = w.units[HERO].hp;
    run(&mut w, &mut res, 5, 5);
    assert_eq!(w.units[HERO].guard, 0.5);
    let (mult, crit) = mult_and_crit(&res, a);
    assert_eq!(hp - w.units[HERO].hp, expected(&w, FOE, HERO, mult, crit), "the hit through guard 0.5");
    assert_eq!(res.scheduled[a].command.staggers[0].factors.guard, w.ruleset.numbers.stagger_guard_mult);
    assert_eq!(fired(&res, HERO, "guarded_hit"), 1);
    run(&mut w, &mut res, 6, 6);
    res.apply_round_end_effects(&mut w);
    assert_eq!(w.units[HERO].held_guard, None, "gone at the round's end");
}

/// 8.2: `ap` 2 at 9 of 10 gives 10; on an enemy-side holder nothing. `grant` until guard_end lasts while the Defend
/// runs. `reveal` shows the next round's plans, then the battle's setting returns.
#[test]
fn ap_grant_and_reveal() {
    let s = session(rules());
    let mut w = s.w.clone();
    w.units[HERO].ap = 9;
    give(&mut w, HERO, "leverage", "hit_taken", json!({}), json!([{"type": "ap", "value": 2}]), "round");
    give(&mut w, FOE, "greed", "hit_dealt", json!({}), json!([{"type": "ap", "value": 2}]), "round");
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "t_jab"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(w.units[HERO].ap, 10, "capped at max AP");
    assert_eq!(fired(&res, FOE, "hit_dealt"), 0, "party side only: nothing to log");

    let mut w = s.w.clone();
    give(&mut w, HERO, "phalanx", "guard_start", json!({}), json!([{"type": "grant", "status": "steadfast", "rounds": 1, "until": "guard_end"}]), "round");
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, own(&w, HERO, "guard"), 1);
    res.prepare_block(&mut w, 1);
    assert!(w.units[HERO].has_status("steadfast"), "granted as the Defend starts");
    run(&mut w, &mut res, 1, 2);
    assert!(w.units[HERO].has_status("steadfast"), "while it runs");
    res.prepare_block(&mut w, 3);
    assert!(!w.units[HERO].has_status("steadfast"), "gone once it ends");

    let mut w = s.w.clone();
    w.units[FOE].hp = 1;
    give(&mut w, HERO, "dead_eye", "kill", json!({}), json!([{"type": "reveal", "rounds": 1}]), "round");
    let base = w.plan_reveal;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, hit(&w, HERO, FOE, "t_jab"), 1);
    run(&mut w, &mut res, 1, 1);
    for (round, shown) in [(2, true), (3, base)] {
        res.finish_round(&mut w);
        trigger::round_start(&mut res, &mut w);
        assert_eq!((res.current_round, w.plan_reveal), (round, shown));
    }
}

/// 8.1: `round_start` and `round_end` fire for every holder on the field; a trigger `move` pushes the other unit away
/// from the holder (FM9, rule forced_move) and its cells follow the trigger's.
#[test]
fn round_events_and_a_trigger_move() {
    let s = session(timeline(&[TRIGGERS, POISE, FORCED_MOVE]));
    let mut w = s.w.clone();
    count(&mut w, HERO, "round_start", "round");
    count(&mut w, FOE, "round_end", "round");
    give(&mut w, HERO, "shove", "hit_taken", json!({}), json!([{"type": "move", "kind": "push", "tiles": 1, "to": "other"}]), "round");
    let mut res = Resolver::new(BLOCKS, 1);
    trigger::round_start(&mut res, &mut w);
    assert_eq!(fired(&res, HERO, "round_start"), 1);
    book(&mut res, FOE, hit(&w, FOE, HERO, "t_jab"), 1);
    run(&mut w, &mut res, 1, BLOCKS);
    assert_eq!(w.pos(FOE), V2::new(6, 4), "pushed 1 tile away from the hero");
    let f = res.triggers.fired.iter().find(|f| f.on == "hit_taken").expect("the move");
    assert_eq!((f.moves.len(), f.moves[0].tiles, f.moves[0].by), (1, 1, Some(HERO)));
    res.apply_round_end_effects(&mut w);
    assert_eq!(fired(&res, FOE, "round_end"), 1);
}

/// In a session each trigger effect gets an EVENT cell (subtype Trigger: the holder, the other unit, the event) and the
/// metrics stay balanced; with the rule off (and in classic) the same holders fire, log and draw nothing.
#[test]
fn the_session_logs_triggers_and_off_changes_nothing() {
    let mut s = session(rules());
    give(&mut s.w, FOE, "spite", "hit_taken", json!({}), json!([{"type": "damage", "value": 7, "to": "other"}]), "round");
    s.attach_metrics(BattleMetrics::new(), true);
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_jab"), FOE), true), "");
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
    let cells: Vec<_> = s.log.iter().filter(|e| e.subtype == "Trigger").collect();
    assert_eq!(cells.len(), 1, "one effect, one cell");
    assert_eq!((cells[0].category, cells[0].actor.as_str(), cells[0].target.as_str(), cells[0].status.as_str()), ("EVENT", "p_foe", "p_hero", "hit_taken"));
    assert!(s.metrics.as_ref().unwrap().fail_lines.is_empty(), "{:?}", s.metrics.as_ref().unwrap().fail_lines);

    for rules in [timeline(&[POISE]), Ruleset::classic()] {
        let s = session(rules.clone());
        let mut w = s.w.clone();
        let mut plain = s.w.clone();
        give(&mut w, FOE, "spite", "hit_taken", json!({}), json!([{"type": "damage", "value": 7, "to": "other"}]), "none");
        give(&mut w, ALLY, "penance", "ally_hit", json!({"range": 1}), json!([{"type": "redirect", "pct": 100}]), "none");
        give(&mut w, HERO, "brace", "hit_taken", json!({}), json!([{"type": "guard", "mitigation": 0.5}]), "none");
        let mut res = Resolver::new(BLOCKS, 1);
        let mut res_plain = Resolver::new(BLOCKS, 1);
        for (w, res) in [(&mut w, &mut res), (&mut plain, &mut res_plain)] {
            book(res, HERO, hit(w, HERO, FOE, "t_jab"), 1);
            book(res, FOE, hit(w, FOE, HERO, "t_jab"), 2);
            run(w, res, 1, 2);
            res.apply_round_end_effects(w);
        }
        assert!(res.triggers.fired.is_empty() && res.log.iter().all(|l| !l.starts_with("[Trigger]")), "{}", rules.id);
        assert_eq!(res.log, res_plain.log, "{}: the same log", rules.id);
        let hp = |w: &World| w.units.iter().map(|u| u.hp).collect::<Vec<_>>();
        assert_eq!(hp(&w), hp(&plain), "{}: the same HP", rules.id);
        assert_eq!(res.rng.randi(), res_plain.rng.randi(), "{}: no new draw", rules.id);
        assert_eq!(w.units[HERO].held_guard, None);
    }
}
