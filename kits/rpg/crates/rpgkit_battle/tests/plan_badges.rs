//! RULESET1-V on a scratch catalog built here (no shipped data): enemy plan visibility (RULESET1 section 10, rule
//! `plan_badges`) and the round intent in tells and threat intel (RULESET1_INTENT UI5). Expectations come from the
//! rules: a tell names its unit's round intent and round target (V1, V5); `round_intents` covers both sides (V5); an
//! impact badge sits at the impact block of every live action but a move, hit for an Attack or Area, else other (V1);
//! the battle's `plan_reveal` reveals every enemy (V2); a foresight `range` reveals every enemy within it (V4); the
//! trace adds its keys only under the rule. Classic (and the rule switched off) keeps today's tells and trace shape.

use indexmap::IndexMap;
use rpgkit_battle::command::CmdKind;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent::{self, Intent, HIT, OTHER};
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::round_intent::{self, ManeuverKind, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, PLAN_BADGES, ROUND_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

/// Engine skills and a few test skills: an Assault cut, a Guard brace, a Support haste, a Maneuver attack.
fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_cut", "name": "Cut", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0}),
        json!({"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3}),
        json!({"id": "t_haste", "name": "Quicken", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY_OR_SELF", "grant_status": "haste", "status_rounds": 3}),
        json!({"id": "t_hamstring", "name": "Hamstring Cut", "kind": "ATTACK", "round_intent": "maneuver", "level": 1, "blocks": 1, "power_mult": 0.8, "dash_tiles": 3}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_ally", "party"), unit("t_foe", "enemy"), unit("t_foe_b", "enemy")]}))
}

fn ruleset(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// A timeline with the round intent and the plan badges.
fn timeline() -> Ruleset {
    ruleset(&[ROUND_INTENT, PLAN_BADGES])
}

/// Hero, ally, foe, foe_b (unit ids 0..4) under `ruleset`, moved off to the corners (tests place them).
fn session(ruleset: Ruleset, plan_reveal: bool) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["t_hero", "t_ally"], "enemies": ["t_foe", "t_foe_b"], "total_blocks": BLOCKS, "unit_ap": UNIT_AP, "ap_regen": 0, "max_rounds": 8});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    def.plan_reveal = plan_reveal;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(0, 0)), (ALLY, V2::new(0, 8)), (FOE, V2::new(11, 0)), (FOE_B, V2::new(11, 8))] {
        place(&mut s.w, c, cell);
    }
    s
}

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn add(plan: &mut Plan, w: &World, c: UnitId, step: rpgkit_battle::plan::Step) {
    assert_eq!(plan.add_step(w, c, step, false), "", "step for {}", w.units[c].name);
}

/// Replaces the round's enemy actions with `plan` (the session stays in planning); a scheduling, so the foresight
/// ranges are measured now (V4).
fn schedule_enemies(s: &mut Session, plan: &Plan) {
    s.resolver.scheduled.clear();
    plan.commit(&s.w, &mut s.resolver);
    s.measure_reveals();
}

/// The badges V1 asks for, read off the plan entries: every entry but a move or Rest (user 2026-10-26: "No badge on
/// Rest"), at its impact block, hit for an Attack or Area, else other.
fn expected_badges(w: &World, plan: &Plan, c: UnitId) -> Vec<(i32, &'static str)> {
    plan.entries_for(w, c)
        .iter()
        .filter(|e| !matches!(skill::kind(&w.cat.skills[e.skill]), Kind::Move | Kind::Rest))
        .map(|e| (e.end_block(w), if skill::kind(&w.cat.skills[e.skill]) == Kind::Attack { HIT } else { OTHER }))
        .collect()
}

/// The standard round: foe (Assault, the hero) steps next to the hero, cuts and rests; foe_b (Guard, itself) braces and
/// rests every block left (a Rest-filled round); the hero (Assault, the foe) rests; the ally (Support, no target set)
/// quickens the hero. Returns the enemy plan.
fn standard_round(s: &mut Session) -> Plan {
    let w = &mut s.w;
    place(w, HERO, V2::new(4, 4));
    place(w, FOE, V2::new(5, 3));
    place(w, FOE_B, V2::new(9, 6));
    place(w, ALLY, V2::new(3, 4));
    let picks = [(FOE, RoundIntent::Assault, Some(RoundTarget::Unit(HERO))), (FOE_B, RoundIntent::Guard, Some(RoundTarget::Myself)), (HERO, RoundIntent::Assault, Some(RoundTarget::Unit(FOE))), (ALLY, RoundIntent::Support, None)];
    let on = round_intent::on(w);
    for (c, i, t) in picks.into_iter().filter(|_| on) {
        round_intent::set_intent(w, c, i).expect("intent");
        if let Some(t) = t {
            round_intent::set_target(w, c, t).expect("target");
        }
    }
    let mut enemy = Plan::new();
    add(&mut enemy, w, FOE, PlanStep::mv(sid(w, "move"), V2::new(0, 1), 1));
    add(&mut enemy, w, FOE, PlanStep::engage(sid(w, "t_cut"), HERO));
    add(&mut enemy, w, FOE, PlanStep::self_skill(sid(w, "rest")));
    add(&mut enemy, w, FOE_B, PlanStep::self_skill(sid(w, "t_brace")));
    while enemy.compile(w, FOE_B).blocks_left() > 0 {
        add(&mut enemy, w, FOE_B, PlanStep::self_skill(sid(w, "rest")));
    }
    let rest = PlanStep::self_skill(sid(&s.w, "rest"));
    let haste = PlanStep::ally(sid(&s.w, "t_haste"), HERO);
    schedule_enemies(s, &enemy);
    let mut party = std::mem::take(&mut s.plan);
    add(&mut party, &s.w, HERO, rest);
    add(&mut party, &s.w, ALLY, haste);
    s.plan = party;
    enemy
}

fn record(list: &[Intent], c: UnitId) -> &Intent {
    list.iter().find(|r| r.unit == c).unwrap_or_else(|| panic!("a record for unit {c}"))
}

/// V1, V5, AI4: under `plan_badges` each enemy tell carries its round intent and round target (the set one) with the
/// first action; badges sit at every live impact but a move or Rest (user 2026-10-26: "No badge on Rest"); a Defend is
/// other; nothing is revealed without foresight, enrage or the reveal.
#[test]
fn a_timeline_tell_names_the_intent_and_target_and_badges_the_impacts() {
    let mut s = session(timeline(), false);
    let enemy = standard_round(&mut s);
    let tells = s.enemy_intents();
    assert_eq!(tells.iter().map(|r| r.unit).collect::<Vec<_>>(), [FOE, FOE_B], "intents stays enemy-only");
    let foe = record(&tells, FOE);
    assert_eq!((foe.round_intent, foe.round_target, foe.maneuver), (Some(RoundIntent::Assault), Some(RoundTarget::Unit(HERO)), None));
    assert_eq!(foe.line.kind, "move", "the tell's kind stays the first action's command kind");
    let entries = enemy.entries_for(&s.w, FOE);
    let cut = entries.iter().find(|e| e.skill == sid(&s.w, "t_cut")).expect("the cut");
    assert_eq!(foe.impacts, [(cut.end_block(&s.w), HIT)], "the cut alone: the move and the Rest have no badge");
    assert_eq!(foe.impacts, expected_badges(&s.w, &enemy, FOE));
    assert!(!foe.revealed && foe.plan.is_empty() && foe.readers.is_empty(), "badges show without revealing the plan");
    let guard = record(&tells, FOE_B);
    assert_eq!((guard.round_intent, guard.round_target), (Some(RoundIntent::Guard), Some(RoundTarget::Myself)));
    let guard_entries = enemy.entries_for(&s.w, FOE_B);
    let rests = guard_entries.iter().filter(|e| e.skill == sid(&s.w, "rest")).count();
    assert_eq!(rests as i32, BLOCKS - guard_entries[0].blocks(&s.w), "a Rest-filled round: every block after the brace rests");
    assert_eq!(guard.impacts, [(guard_entries[0].end_block(&s.w), OTHER)], "the brace is other; its Rest blocks have no badge");
    assert_eq!(guard.impacts, expected_badges(&s.w, &enemy, FOE_B));
    assert_eq!(guard.line.kind, "guard", "a Defend's kind is 'guard' whatever the intent");
    // a cancelled action is not live: its badge goes
    let cut = s.resolver.scheduled.iter().position(|a| a.unit == FOE && matches!(a.command.kind, CmdKind::Attack { .. })).expect("the cut");
    s.resolver.scheduled[cut].cancelled = true;
    assert!(record(&s.enemy_intents(), FOE).impacts.iter().all(|b| b.1 != HIT), "a cancelled cut has no badge");
}

/// V5, UI5: `round_intents` covers every active unit, party first (from their plans: always revealed), then the
/// enemies as their tells; a party target not set comes from the first step that acts on one; after the plan is
/// confirmed the party records read the committed actions (the same lines).
#[test]
fn round_intents_cover_both_sides() {
    let mut s = session(timeline(), false);
    standard_round(&mut s);
    let all = s.round_intents();
    assert_eq!(all.iter().map(|r| r.unit).collect::<Vec<_>>(), [HERO, ALLY, FOE, FOE_B]);
    let hero = record(&all, HERO);
    assert_eq!((hero.round_intent, hero.round_target), (Some(RoundIntent::Assault), Some(RoundTarget::Unit(FOE))));
    assert!(hero.revealed && hero.readers.is_empty());
    assert_eq!(hero.plan.len(), s.plan.entries_for(&s.w, HERO).len());
    assert_eq!(hero.impacts, expected_badges(&s.w, &s.plan, HERO));
    let ally = record(&all, ALLY);
    assert_eq!((ally.round_intent, ally.round_target), (Some(RoundIntent::Support), Some(RoundTarget::Unit(HERO))), "the first step's target");
    assert_eq!(ally.line.kind, "support");
    assert_eq!(record(&all, FOE), record(&s.enemy_intents(), FOE), "an enemy record is its tell");
    let planned: Vec<_> = [HERO, ALLY].iter().map(|&c| record(&all, c).plan.clone()).collect();
    assert!(s.confirm_plan());
    let after = s.round_intents();
    let committed: Vec<_> = [HERO, ALLY].iter().map(|&c| record(&after, c).plan.clone()).collect();
    assert_eq!(committed, planned, "the committed actions read as the plan did");
    // a Maneuver round names its kind (MV11: Engage until one is picked)
    let mut m = session(timeline(), false);
    round_intent::set_intent(&mut m.w, ALLY, RoundIntent::Maneuver).unwrap();
    assert_eq!(record(&m.round_intents(), ALLY).maneuver, Some(ManeuverKind::Engage));
    round_intent::set_maneuver(&mut m.w, ALLY, ManeuverKind::Disengage).unwrap();
    let rec = m.round_intents();
    assert_eq!((record(&rec, ALLY).round_intent, record(&rec, ALLY).maneuver), (Some(RoundIntent::Maneuver), Some(ManeuverKind::Disengage)));
    assert_eq!(record(&rec, HERO).round_intent, None, "no intent picked yet");
}

/// V2: with the battle's `plan_reveal` every enemy is revealed with its whole plan (each live action with its windup,
/// impact and recovery); classic ignores the flag.
#[test]
fn plan_reveal_shows_every_enemy_plan() {
    for (rules, on) in [(timeline(), true), (Ruleset::classic(), false)] {
        let mut s = session(rules, true);
        assert!(s.w.plan_reveal, "the encounter's flag reaches the battle");
        let enemy = standard_round(&mut s);
        for r in s.enemy_intents() {
            assert_eq!(r.revealed, on, "{}", s.w.units[r.unit].name);
            if on {
                let actions = intent::actions_of(r.unit, &s.resolver.scheduled);
                assert_eq!(r.plan.len(), enemy.entries_for(&s.w, r.unit).len());
                assert_eq!(r.plan, actions.iter().map(|a| intent::line_of(&s.w, a)).collect::<Vec<_>>(), "the whole plan");
            } else {
                assert!(r.plan.is_empty());
            }
        }
    }
}

/// V4 (tasks 1.4): a foresight passive's `range` (the widest one, at most 8; gear never grants foresight); with range N
/// the holder reveals every enemy within N tiles at planning time, on top of today's rule; classic, and a holder
/// without a range, keep today's rule (only enemies targeting the holder).
#[test]
fn a_foresight_range_reveals_every_enemy_within_it() {
    let cat = Arc::new(catalog());
    let mut u = rpgkit_rules::build_unit(&cat, cat.units.get("t_hero").unwrap(), UNIT_AP);
    u.apply_effects(&cat, &[json!({"type": "foresight", "range": 4})], "passive:hawk_eye", false);
    u.apply_effects(&cat, &[json!({"type": "foresight", "range": 2})], "passive:owl", false);
    assert_eq!((u.foresight_source.as_str(), u.foresight_range), ("hawk_eye", 4), "the first source, the widest range");
    u.apply_effects(&cat, &[json!({"type": "foresight", "range": 12})], "gear:lens", true);
    assert_eq!(u.foresight_range, 4, "gear grants no foresight");
    u.apply_effects(&cat, &[json!({"type": "foresight", "range": 12})], "passive:far", false);
    assert_eq!(u.foresight_range, rpgkit_rules::unit::FORESIGHT_RANGE_MAX, "clamped to the schema's maximum");
    for (rules, range, reveals) in [(timeline(), 4, true), (Ruleset::classic(), 4, false), (timeline(), 0, false)] {
        let mut s = session(rules, false);
        let w = &mut s.w;
        place(w, HERO, V2::new(2, 4));
        place(w, ALLY, V2::new(0, 0));
        place(w, FOE, V2::new(6, 4));
        place(w, FOE_B, V2::new(7, 4));
        w.units[HERO].foresight_source = "hawk_eye".into();
        w.units[HERO].foresight_range = range;
        assert_eq!((w.dist(HERO, FOE), w.dist(HERO, FOE_B)), (4, 5));
        // RULESET1-AI: the session's enemy AI picked intents at the round start; these Braces need Guard
        for c in [FOE, FOE_B] {
            if round_intent::on(w) {
                round_intent::set_intent(w, c, RoundIntent::Guard).expect("intent");
            }
        }
        let mut enemy = Plan::new();
        add(&mut enemy, w, FOE, PlanStep::self_skill(sid(w, "t_brace")));
        add(&mut enemy, w, FOE_B, PlanStep::self_skill(sid(w, "t_brace")));
        schedule_enemies(&mut s, &enemy);
        let tells = s.enemy_intents();
        let foe = record(&tells, FOE);
        assert_eq!(foe.revealed, reveals, "range {range}: the foe 4 tiles away");
        assert_eq!(foe.readers, if reveals { vec![HERO] } else { Vec::new() });
        assert_eq!(foe.plan.len(), if reveals { 1 } else { 0 });
        assert!(!record(&tells, FOE_B).revealed, "5 tiles is out of range");
        // the RULESET1-V review: the range is measured when the plan is scheduled, so units moving later (the round's
        // playback) neither hide the foe it revealed nor reveal the one it did not
        place(&mut s.w, HERO, V2::new(2, 8));
        place(&mut s.w, FOE_B, V2::new(3, 8));
        assert!(s.w.dist(HERO, FOE) > range && s.w.dist(HERO, FOE_B) <= 1, "the hero now stands by foe_b, far from foe");
        for (when, tells) in [("tells", s.enemy_intents()), ("threat intel", s.round_intents())] {
            if when == "threat intel" && !intent::badges_on(&s.w) {
                assert!(tells.is_empty(), "classic has no round intents");
                continue;
            }
            let (a, b) = (record(&tells, FOE), record(&tells, FOE_B));
            assert_eq!((a.revealed, a.readers.clone()), (reveals, if reveals { vec![HERO] } else { Vec::new() }), "{when}: range {range}: the reveal at scheduling holds");
            assert!(!b.revealed, "{when}: range {range}: a foe that came into range after scheduling stays hidden");
        }
        // a new scheduling measures again
        s.measure_reveals();
        let tells = s.enemy_intents();
        assert_eq!((record(&tells, FOE).revealed, record(&tells, FOE_B).revealed), (false, reveals), "range {range}: measured again at the next scheduling");
    }
}

/// V5, 3.3: the trace adds [intent, Maneuver kind, round target, impacts] to each record and the recovery to each line
/// only under `plan_badges`; classic keeps the Godot shape (8 fields a record, 7 a line) and fills no round field;
/// the round intent without `plan_badges` fills none either (the Arena's switch).
#[test]
fn the_trace_keys_come_only_with_the_rule() {
    let mut s = session(timeline(), false);
    standard_round(&mut s);
    let trace = intent::trace(&s.w, &s.enemy_intents());
    let foe = trace.as_array().unwrap()[0].as_array().unwrap().clone();
    assert_eq!(foe.len(), 12);
    assert_eq!(foe[1].as_array().unwrap().len(), 8, "a line adds its recovery");
    assert_eq!((foe[8].clone(), foe[9].clone(), foe[10].clone()), (json!("assault"), json!(""), json!(["unit", "t_hero", -1, -1])));
    let impacts = record(&s.enemy_intents(), FOE).impacts.clone();
    assert_eq!(foe[11], json!(impacts.iter().map(|(b, k)| json!([b, k])).collect::<Vec<_>>()));
    for rules in [Ruleset::classic(), ruleset(&[ROUND_INTENT])] {
        let mut c = session(rules.clone(), false);
        standard_round(&mut c);
        let tells = c.enemy_intents();
        assert!(tells.iter().all(|r| r.round_intent.is_none() && r.round_target.is_none() && r.maneuver.is_none() && r.impacts.is_empty()), "{:?}", rules.rules);
        assert!(c.round_intents().is_empty(), "no round intents without plan_badges");
        let t = intent::trace(&c.w, &tells);
        for rec in t.as_array().unwrap() {
            assert_eq!(rec.as_array().unwrap().len(), 8, "the Godot record shape");
            assert_eq!(rec[1].as_array().unwrap().len(), 7, "the Godot line shape");
        }
    }
}

/// RULESET1-U slice 5 (V2's board path): a revealed walk is drawn as the kit would walk it now (`walk_preview`): from
/// its planned start through each step, stopping before a taken or unwalkable tile and at the board's edge; the real
/// walk (the planned move, resolved) ends on the preview's last tile.
#[test]
fn a_walk_preview_stops_where_the_walk_would() {
    let mut s = session(timeline(), false);
    let west = [V2::new(-1, 0); 3];
    place(&mut s.w, FOE, V2::new(6, 3));
    place(&mut s.w, HERO, V2::new(0, 0));
    place(&mut s.w, ALLY, V2::new(0, 8));
    let free = rpgkit_battle::command::walk_preview(&s.w, FOE, &west, V2::new(6, 3));
    assert_eq!(free, [V2::new(6, 3), V2::new(5, 3), V2::new(4, 3), V2::new(3, 3)], "a free path: every tile");
    place(&mut s.w, ALLY, V2::new(4, 3));
    let cut = rpgkit_battle::command::walk_preview(&s.w, FOE, &west, V2::new(6, 3));
    assert_eq!(cut, [V2::new(6, 3), V2::new(5, 3)], "stops before the taken tile");
    // the real walk of the same steps ends there too
    let mut w = s.w.clone();
    for d in west {
        if !w.move_unit(FOE, d) {
            break;
        }
    }
    assert_eq!(w.pos(FOE), *cut.last().unwrap());
    place(&mut s.w, FOE, V2::new(1, 3));
    place(&mut s.w, ALLY, V2::new(0, 8));
    let edge = rpgkit_battle::command::walk_preview(&s.w, FOE, &west, rpgkit_battle::grid::NO_CELL);
    assert!(edge.iter().all(|c| s.w.grid.is_valid(*c)) && edge.len() <= 2, "the board's edge stops it: {edge:?}");
}
