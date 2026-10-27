//! RULESET1-IN on a scratch catalog built here (no shipped data): the round intent rules of RULESET1_INTENT 2-6 under
//! the rule `round_intent` (and `flank` for the flank bonus). Expectations come from the rules: the intent filter and
//! one target (IN5, TG1, MM1), persistence and intent-bound buffs (IN2, IN6), Rest paid at round end (RS2, RS4), a lost
//! target (TG4), facing and flanks (MV1-MV3), carried Engage and Disengage moves (MV4-MV9), after-moves (AS3) and
//! Guard stealth (GD5). Classic keeps today's rules (the parity suites; the classic cases here).

use indexmap::IndexMap;
use rpgkit_battle::command::{Carry, CmdKind};
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, ManeuverKind, RoundIntent, RoundTarget};
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, FLANK, ROUND_INTENT, TIMELINE};
use rpgkit_rules::{status, Stance};
use serde_json::json;
use std::sync::Arc;

const BLOCKS: i32 = 6;
const UNIT_AP: i32 = 10;

/// Engine skills and the test skills: Assault attacks (a cut, a heavy, a flank exploit, an area), Maneuver attacks with
/// carried moves (`dash_tiles` 3 and 2), Guard skills (riposte, stealth, an `intent_buff` grant), a Support haste.
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
        json!({"id": "t_heavy", "name": "Heavy", "kind": "ATTACK", "level": 3, "blocks": 2, "power_mult": 2.0}),
        json!({"id": "t_backstab", "name": "Backstab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "exploit": [{"status": "flanked", "power_mult": 2.0}]}),
        json!({"id": "t_burst", "name": "Burst", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 1.0, "area": "PLUS"}),
        json!({"id": "t_hamstring", "name": "Hamstring Cut", "kind": "ATTACK", "round_intent": "maneuver", "level": 2, "blocks": 1, "power_mult": 0.8, "dash_tiles": 3}),
        json!({"id": "t_slip", "name": "Slip Strike", "kind": "ATTACK", "round_intent": "maneuver", "level": 2, "blocks": 2, "power_mult": 0.9, "dash_tiles": 2}),
        json!({"id": "t_brace", "name": "Brace", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.3, "grant_status": "riposte", "status_rounds": 2}),
        json!({"id": "t_fade", "name": "Fade", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.2, "grant_status": "stealth", "status_rounds": 2}),
        json!({"id": "t_bulwark", "name": "Bulwark", "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.1, "intent_buff": true, "extra_grants": [{"status": "fortify", "rounds": 3, "magnitude": 5}]}),
        json!({"id": "t_haste", "name": "Quicken", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY_OR_SELF", "grant_status": "haste", "status_rounds": 3}),
    ];
    let unit = |id: &str, side: &str, default_intent: &str| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5, "default_intent": default_intent})
    };
    Catalog::from_value(&json!({"skills": skills, "units": [
        unit("t_hero", "party", ""), unit("t_ally", "party", "guard"), unit("t_foe", "enemy", ""), unit("t_foe_b", "enemy", "guard")
    ]}))
}

/// A timeline ruleset with the round intent and the flank bonus (numbers at their defaults).
fn timeline() -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": [ROUND_INTENT, FLANK]}).as_object().unwrap())
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
    s
}

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE_B: UnitId = 3;

fn place(w: &mut World, c: UnitId, cell: V2) {
    w.grid.positions[c] = Some(cell);
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn intent(w: &mut World, c: UnitId, i: RoundIntent, t: RoundTarget) {
    round_intent::set_intent(w, c, i).expect("intent");
    round_intent::set_target(w, c, t).expect("target");
}

/// Runs blocks `from..=to` of `res` on `w` (start, then every landing action).
fn run(w: &mut World, res: &mut Resolver, from: i32, to: i32) {
    for b in from..=to {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

/// Commits `plan` into a fresh resolver.
fn commit(w: &World, plan: &Plan) -> Resolver {
    let mut res = Resolver::new(BLOCKS, 7);
    plan.commit(w, &mut res);
    res
}

/// IN3, IN5, TG1, MM1: under an intent an Assault round takes Assault skills on its one target (no stance change, no
/// switch AP, matchup 1.0) and the neutral plain move and Rest; another intent's skill and another target are refused.
/// A unit without an intent, and classic, accept all of it.
#[test]
fn an_intent_limits_the_skills_and_the_target() {
    for (ruleset, on) in [(timeline(), true), (Ruleset::classic(), false)] {
        let mut s = session(ruleset);
        let w = &mut s.w;
        place(w, HERO, V2::new(3, 4));
        place(w, FOE, V2::new(5, 4));
        place(w, FOE_B, V2::new(4, 5));
        let steps = |w: &World| {
            vec![
                ("move", PlanStep::mv(sid(w, "move"), V2::new(1, 0), 1)),
                ("assault skill", PlanStep::engage(sid(w, "t_cut"), FOE)),
                ("guard skill", PlanStep::self_skill(sid(w, "t_fade"))),
                ("other target", PlanStep::engage(sid(w, "t_cut"), FOE_B)),
                ("rest", PlanStep::self_skill(sid(w, "rest"))),
            ]
        };
        // no intent yet (or classic): nothing is filtered
        let mut plan = Plan::new();
        for (name, step) in steps(w) {
            assert_eq!(plan.add_step(w, HERO, step, true), "", "on {on}, no intent: {name}");
        }
        if !on {
            assert!(round_intent::set_intent(w, HERO, RoundIntent::Assault).is_err(), "classic has no intents");
            continue;
        }
        intent(w, HERO, RoundIntent::Assault, RoundTarget::Unit(FOE));
        let mut plan = Plan::new();
        for (name, step) in steps(w) {
            let p = plan.add_step(w, HERO, step, true);
            match name {
                "guard skill" => assert_eq!(p, "Only Assault skills this round.", "{name}"),
                "other target" => assert_eq!(p, format!("One target per round: {}.", w.units[FOE].name), "{name}"),
                _ => assert_eq!(p, "", "{name}"),
            }
        }
        let entries = plan.entries_for(w, HERO);
        let cut = entries.iter().find(|e| e.skill == sid(w, "t_cut")).unwrap();
        assert_eq!((cut.switch_ap, cut.stance_before, cut.stance_after), (0, Stance::Neutral, Stance::Neutral), "IN3");
        assert!(!entries.iter().any(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Target), "TG3: no Switch Target");
        assert_eq!(Stance::Neutral.matchup(Stance::Neutral), 1.0);
        // the intent table: Assault never targets self or an ally
        assert!(round_intent::set_target(w, HERO, RoundTarget::Myself).is_err());
        assert!(round_intent::set_target(w, HERO, RoundTarget::Unit(ALLY)).is_err());
        assert!(round_intent::set_target(w, HERO, RoundTarget::Cell(V2::new(6, 6))).is_ok(), "an area opener may aim at a tile");
    }
}

/// IN2, IN6: an intent change at a round start drops the intent-bound instances (riposte, stealth, an `intent_buff`
/// grant to its own user) and keeps every other instance with its rounds (an ally's haste, a debuff); keeping the
/// intent drops nothing. The intent persists into the next round.
#[test]
fn an_intent_change_drops_only_intent_bound_buffs() {
    let mut s = session(timeline());
    place(&mut s.w, HERO, V2::new(2, 4));
    place(&mut s.w, ALLY, V2::new(2, 5));
    s.set_round_intent(HERO, "guard").unwrap();
    s.set_round_target(HERO, RoundTarget::Myself).unwrap();
    s.set_round_intent(ALLY, "support").unwrap();
    s.set_round_target(ALLY, RoundTarget::Unit(HERO)).unwrap();
    for id in ["t_brace", "t_fade", "t_bulwark"] {
        assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::self_skill(sid(&s.w, id)), false), "", "{id}");
    }
    assert_eq!(s.plan.add_step(&s.w, ALLY, PlanStep::ally(sid(&s.w, "t_haste"), HERO), false), "");
    status::queue(&mut s.w.units[HERO], "weaken", 3, 10, "test");
    assert!(s.confirm_plan());
    while s.phase != Phase::Replay {
        s.step_next();
    }
    s.advance_round();
    assert_eq!(round_intent::intent_of(&s.w, HERO), Some(RoundIntent::Guard), "the intent persists");
    let held = |s: &Session| s.w.units[HERO].st.rounds_left.clone();
    let before = held(&s);
    for st in ["riposte", "stealth", "fortify", "haste", "weaken"] {
        assert!(before.contains_key(st), "{st} is active in round 2");
    }
    let tags: Vec<&String> = s.w.units[HERO].st.intents.keys().collect();
    assert_eq!(tags, ["riposte", "stealth", "fortify"], "bound: the two rows, the intent_buff grant to its user");
    assert!(s.set_round_intent(HERO, "guard").unwrap().is_empty(), "no change, nothing dropped");
    assert_eq!(held(&s), before);
    let lines = s.set_round_intent(HERO, "assault").unwrap();
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert!(lines[0].contains("leaves Guard") && lines[0].contains("Riposte"), "{lines:?}");
    let after = held(&s);
    for st in ["riposte", "stealth", "fortify"] {
        assert!(!s.w.units[HERO].st.has_any(st), "{st} dropped");
    }
    for st in ["haste", "weaken"] {
        assert_eq!(after.get(st), before.get(st), "{st} keeps its rounds");
    }
    // a confirmed round refuses a change
    assert!(s.confirm_plan());
    assert!(s.set_round_intent(HERO, "guard").is_err(), "no mid-round change");
}

/// The AP `c` holds after a round in which it pays a Heavy and then rests `rests` blocks, and the AP once the next
/// round began (round 1 and 2 of a fresh fight; `ap_regen` 0).
fn rest_round(ruleset: Ruleset, rests: i32) -> (i32, i32, i32, i32) {
    let mut s = session(ruleset);
    place(&mut s.w, HERO, V2::new(4, 4));
    place(&mut s.w, FOE, V2::new(5, 4));
    if round_intent::on(&s.w) {
        s.set_round_intent(HERO, "assault").unwrap();
        s.set_round_target(HERO, RoundTarget::Unit(FOE)).unwrap();
    }
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "t_heavy"), FOE), false), "");
    for _ in 0..rests {
        assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::self_skill(sid(&s.w, "rest")), false), "");
    }
    let heavy = s.plan.entries_for(&s.w, HERO)[0].total_ap();
    let start = s.w.units[HERO].ap;
    assert!(s.confirm_plan());
    while s.phase != Phase::Replay {
        s.step_next();
    }
    let end = s.w.units[HERO].ap;
    s.advance_round();
    (start, heavy, end, s.w.units[HERO].ap)
}

/// RS1, RS2, RS4: under the round intent a landed Rest gives no AP at once; the round end pays
/// `min(rest_ap_cap, floor(blocks x rest_ap_per_block))` after `ap_regen`; classic Rest pays `recover_ap` on landing.
#[test]
fn rest_pays_at_round_end() {
    let n = timeline().numbers;
    for rests in [3, 4] {
        let (start, heavy, end, next) = rest_round(timeline(), rests);
        assert_eq!(end, start - heavy, "{rests} Rest blocks: no AP mid-round");
        let gain = ((rests as f64 * n.rest_ap_per_block).floor() as i32).min(n.rest_ap_cap);
        assert_eq!(gain, round_intent::rest_gain(&n, rests));
        assert_eq!(next, UNIT_AP.min(end + gain), "{rests} Rest blocks pay {gain} at round end");
    }
    assert_eq!(round_intent::rest_gain(&n, 6), n.rest_ap_cap, "capped");
    let (start, heavy, end, next) = rest_round(Ruleset::classic(), 3);
    let recover = 2;
    let mut want = start - heavy;
    for _ in 0..3 {
        want = UNIT_AP.min(want + recover);
    }
    assert_eq!(end, want, "classic: each Rest pays on landing");
    assert_eq!(next, end, "classic: nothing more at round end (ap_regen 0)");
}

/// TG4: the target downed before the third command: it and every later command of the unit become Rest blocks,
/// unpaid (the AP spent is the first two commands').
#[test]
fn a_lost_target_turns_the_rest_of_the_round_to_rest() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, HERO, V2::new(4, 4));
    place(w, FOE, V2::new(5, 4));
    intent(w, HERO, RoundIntent::Assault, RoundTarget::Unit(FOE));
    let mut plan = Plan::new();
    for _ in 0..4 {
        assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, "t_cut"), FOE), false), "");
    }
    let cut_ap = plan.entries_for(w, HERO)[0].total_ap();
    let mut res = commit(w, &plan);
    let ap0 = w.units[HERO].ap;
    run(w, &mut res, 1, 2);
    w.units[FOE].hp = 0;
    run(w, &mut res, 3, BLOCKS);
    let kinds: Vec<bool> = res.scheduled.iter().map(|a| matches!(a.command.kind, CmdKind::Rest { .. })).collect();
    assert_eq!(kinds, [false, false, true, true], "commands 3 and 4 rest");
    assert_eq!(w.units[HERO].ap, ap0 - 2 * cut_ap, "the Rest blocks pay nothing");
    assert_eq!(w.round_states[&HERO].rest_blocks, 2, "and count as Rest blocks (RS2)");
}

/// MV1-MV3: T engaged with E to its east (T's target, facing east); A west and north flank, north-east does not; when
/// T's target is A, A does not flank. The bonus (rule `flank`): `exploit` `flanked` matches and no riposte answers.
#[test]
fn flanks_come_from_beside_or_behind_an_engaged_unit() {
    let mut s = session(timeline());
    let w = &mut s.w;
    assert_eq!((round_intent::facing(w, HERO), round_intent::facing(w, FOE)), (V2::new(1, 0), V2::new(-1, 0)), "battle start: party right, foes left");
    let t = FOE;
    let t_at = V2::new(5, 4);
    place(w, t, t_at);
    place(w, ALLY, t_at + V2::new(1, 0));
    intent(w, t, RoundIntent::Assault, RoundTarget::Unit(ALLY));
    assert_eq!(round_intent::facing(w, t), V2::new(1, 0), "T faces its target");
    assert_eq!(round_intent::engaged_with(w, t), Some(ALLY));
    for (offset, flank) in [(V2::new(-1, 0), true), (V2::new(0, -1), true), (V2::new(1, -1), false)] {
        place(w, HERO, t_at + offset);
        assert_eq!(round_intent::is_flank(w, HERO, t), flank, "A at {offset}");
        assert_eq!(round_intent::flank_bonus(w, HERO, t), flank, "the bonus under flank");
    }
    let mut off = w.clone();
    off.ruleset.adjust("-flank").unwrap();
    place(&mut off, HERO, t_at + V2::new(-1, 0));
    assert!(round_intent::is_flank(&off, HERO, t) && !round_intent::flank_bonus(&off, HERO, t), "drawn without the bonus");
    // the flank exploit and riposte, through the command: A west flanks; with T's target A it does not
    let backstab = sid(w, "t_backstab");
    let exploit = w.cat.skills[backstab].exploit.clone();
    assert_eq!(skill::exploit_match_when(&exploit, &w.units[t], &[], true).1, 2.0);
    assert_eq!(skill::exploit_match(&exploit, &w.units[t], &[]).1, 1.0, "never a status");
    for (t_target, flank) in [(ALLY, true), (HERO, false)] {
        let mut w = w.clone();
        place(&mut w, HERO, t_at + V2::new(-1, 0));
        round_intent::set_target(&mut w, t, RoundTarget::Unit(t_target)).unwrap();
        status::grant_now(&mut w.units[t], "riposte", 2, 0, "test");
        intent(&mut w, HERO, RoundIntent::Assault, RoundTarget::Unit(t));
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(&w, HERO, PlanStep::engage(backstab, t), false), "");
        let mut res = commit(&w, &plan);
        run(&mut w, &mut res, 1, 1);
        let cmd = &res.scheduled[0].command;
        assert_eq!(cmd.exploited.iter().any(|(_, st)| st == "flanked"), flank, "target {t_target}: exploit");
        assert_eq!(cmd.reactions.is_empty(), flank, "target {t_target}: riposte only from the front");
    }
}

/// MV4, MV5, AS1: Engage with Hamstring Cut (`dash_tiles` 3) on a foe 3 tiles away engaged with an ally: one entry, the
/// carried move inside its block to a flank tile in band, at the skill's AP; a plain move before it stays allowed
/// (classic refuses a dash after a move). The same foe under Assault: an approach (1 AP a tile), then the attack.
#[test]
fn engage_carries_its_move_and_assault_approaches() {
    let mut s = session(timeline());
    let w = &mut s.w;
    let foe_at = V2::new(5, 4);
    place(w, HERO, V2::new(2, 4));
    place(w, FOE, foe_at);
    place(w, ALLY, foe_at + V2::new(0, -1));
    intent(w, FOE, RoundIntent::Assault, RoundTarget::Unit(ALLY));
    intent(w, HERO, RoundIntent::Maneuver, RoundTarget::Unit(FOE));
    let ham = sid(w, "t_hamstring");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(ham, FOE), false), "");
    let entries = plan.entries_for(w, HERO);
    assert_eq!(entries.len(), 1, "no move entry");
    let e = &entries[0];
    let band = skill::reach_for(&w.cat.skills[ham], &w.units[HERO]);
    let d = e.pos_after.dist(foe_at);
    assert!(d >= band.0 && d <= band.1 && !round_intent::in_front(w, FOE, e.pos_after), "a flank tile in band: {}", e.pos_after);
    assert!(e.path.len() as i32 <= w.cat.skills[ham].dash_tiles && e.carry == Carry::Engage(e.pos_after));
    assert_eq!(e.skill_ap, skill::ap_cost_by(&w.cat.skills[ham], Some(&w.units[HERO]), &w.ruleset), "the skill's AP only");
    let mut res = commit(w, &plan);
    let hp = w.units[FOE].hp;
    run(w, &mut res, 1, 1);
    assert_eq!(w.pos(HERO), e.pos_after, "moved at the skill's start");
    assert!(w.units[FOE].hp < hp, "and hit");
    assert!(w.round_states[&HERO].moved);
    // a plain move first stays free under the intent; classic counts the dash as the round's move
    for (ruleset, ok) in [(timeline(), true), (Ruleset::classic(), false)] {
        let mut s2 = session(ruleset);
        let w2 = &mut s2.w;
        place(w2, HERO, V2::new(2, 4));
        place(w2, FOE, foe_at);
        place(w2, ALLY, foe_at + V2::new(0, -1));
        if ok {
            intent(w2, FOE, RoundIntent::Assault, RoundTarget::Unit(ALLY));
            intent(w2, HERO, RoundIntent::Maneuver, RoundTarget::Unit(FOE));
        }
        let mut plan = Plan::new();
        assert_eq!(plan.add_step(w2, HERO, PlanStep::mv(sid(w2, "move"), V2::new(0, 1), 1), false), "");
        let p = plan.add_step(w2, HERO, PlanStep::engage(ham, FOE), false);
        assert_eq!(p.is_empty(), ok, "{p}");
    }
    // Assault on the same foe: approach blocks at 1 AP a tile, then the attack
    let mut s3 = session(timeline());
    let w3 = &mut s3.w;
    place(w3, HERO, V2::new(2, 4));
    place(w3, FOE, foe_at);
    intent(w3, HERO, RoundIntent::Assault, RoundTarget::Unit(FOE));
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w3, HERO, PlanStep::engage(sid(w3, "t_cut"), FOE), false), "");
    let entries = plan.entries_for(w3, HERO);
    let moves: Vec<_> = entries.iter().filter(|e| skill::kind(&w3.cat.skills[e.skill]) == Kind::Move).collect();
    let tiles: i32 = moves.iter().map(|e| e.tiles).sum();
    assert_eq!(tiles, V2::new(2, 4).dist(foe_at) - 1, "approach into band");
    assert!(moves.iter().all(|e| e.skill_ap == e.tiles), "1 AP a tile");
    assert_eq!(skill::kind(&w3.cat.skills[entries.last().unwrap().skill]), Kind::Attack);
}

/// The plan of a Maneuver Engage with Hamstring Cut from `hero_at` on the foe at (5,4) facing its target, the hero.
fn face_on(hero_at: V2) -> (Session, Plan) {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, HERO, hero_at);
    place(w, FOE, V2::new(5, 4));
    intent(w, FOE, RoundIntent::Assault, RoundTarget::Unit(HERO));
    intent(w, HERO, RoundIntent::Maneuver, RoundTarget::Unit(FOE));
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, "t_hamstring"), FOE), false), "");
    (s, plan)
}

/// MV6: Engage face-on in band still moves to a flank tile; a stored tile taken at run time is re-picked by PL4; with no
/// flank tile left the move is skipped and the hit lands only if in band.
#[test]
fn a_carried_move_repicks_or_skips_at_run_time() {
    let hero_at = V2::new(4, 4);
    let (mut s, plan) = face_on(hero_at);
    let stored = plan.entries_for(&s.w, HERO)[0].pos_after;
    assert!(stored != hero_at && !round_intent::in_front(&s.w, FOE, stored) && stored.dist(V2::new(5, 4)) == 1, "face-on in band: a flank tile {stored}");
    // taken at run time: the next PL4 tile
    let w = &mut s.w;
    let mut res = commit(w, &plan);
    place(w, ALLY, stored);
    run(w, &mut res, 1, 1);
    let at = w.pos(HERO);
    assert!(at != stored && at != hero_at && !round_intent::in_front(w, FOE, at) && at.dist(V2::new(5, 4)) == 1, "re-picked {at}");
    // no flank tile left: in band the hit still lands, out of band it does not
    for (start, lands) in [(hero_at, true), (V2::new(3, 4), false)] {
        let (mut s, plan) = face_on(start);
        let w = &mut s.w;
        let mut res = commit(w, &plan);
        let foe_at = V2::new(5, 4);
        // the side tiles taken; the one behind the foe is reachable only around it, beyond the carried move
        place(w, ALLY, foe_at + V2::new(0, -1));
        place(w, FOE_B, foe_at + V2::new(0, 1));
        let hp = w.units[FOE].hp;
        run(w, &mut res, 1, 1);
        assert_eq!(w.pos(HERO), start, "move skipped");
        assert!(res.log.iter().any(|l| l.starts_with("[Dash Blocked]")), "{:?}", res.log);
        assert_eq!(w.units[FOE].hp < hp, lands, "from {start}: hits only in band (foe at {foe_at})");
    }
}

/// MV8: Disengage with two Slip Strikes: both hit from the unit's tile; the last carries the retreat after its hit,
/// within its `dash_tiles` in one block, inside its own blocks. MV9 (TQ22 a): an Engage opener out of range takes a
/// standard plain move first; with the move used it is refused.
#[test]
fn disengage_retreats_after_the_last_hit_and_an_out_of_range_engage_walks_first() {
    let mut s = session(timeline());
    let w = &mut s.w;
    let (hero_at, foe_at) = (V2::new(4, 4), V2::new(5, 4));
    place(w, HERO, hero_at);
    place(w, FOE, foe_at);
    intent(w, HERO, RoundIntent::Maneuver, RoundTarget::Unit(FOE));
    round_intent::set_maneuver(w, HERO, ManeuverKind::Disengage).unwrap();
    let slip = sid(w, "t_slip");
    let mut plan = Plan::new();
    for _ in 0..2 {
        assert_eq!(plan.add_step(w, HERO, PlanStep::engage(slip, FOE), false), "");
    }
    let entries = plan.entries_for(w, HERO);
    assert_eq!((entries[0].carry, entries[0].pos_after), (Carry::None, hero_at), "the first hit stays");
    let Carry::Retreat(cell) = entries[1].carry else { panic!("the last one retreats: {:?}", entries[1].carry) };
    assert!(cell != hero_at && cell.dist(foe_at) > hero_at.dist(foe_at), "away: {cell}");
    let tiles = round_intent::carry_tiles(w, hero_at, w.cat.skills[slip].dash_tiles, &Default::default());
    assert!(tiles.iter().any(|(c, _)| *c == cell), "within its carried move");
    let mut res = commit(w, &plan);
    let hp = w.units[FOE].hp;
    let impact = entries[1].end_block(w);
    run(w, &mut res, 1, impact - 1);
    assert_eq!(w.pos(HERO), hero_at, "still in place before the last impact");
    let mid = w.units[FOE].hp;
    run(w, &mut res, impact, impact);
    assert_eq!(w.pos(HERO), cell, "retreated at the impact");
    assert!(mid < hp && w.units[FOE].hp < mid, "both hits landed");
    // MV9: out of range, a plain move first
    let mut s = session(timeline());
    let w = &mut s.w;
    let foe_at = V2::new(8, 4);
    place(w, HERO, V2::new(1, 4));
    place(w, FOE, foe_at);
    place(w, ALLY, foe_at + V2::new(0, -1));
    intent(w, FOE, RoundIntent::Assault, RoundTarget::Unit(ALLY));
    intent(w, HERO, RoundIntent::Maneuver, RoundTarget::Unit(FOE));
    let ham = sid(w, "t_hamstring");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(ham, FOE), false), "");
    let entries = plan.entries_for(w, HERO);
    let (last, moves) = entries.split_last().unwrap();
    assert!(!moves.is_empty() && moves.iter().all(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Move && e.skill_ap == e.tiles), "a plain move first");
    assert!(moves.len() <= 2, "the standard move's blocks");
    assert_eq!(last.skill, ham);
    assert!(!round_intent::in_front(w, FOE, last.pos_after) && last.pos_after.dist(foe_at) == 1);
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::mv(sid(w, "move"), V2::new(1, 0), 1), false), "");
    assert!(plan.add_step(w, HERO, PlanStep::engage(ham, FOE), false).contains("beyond"), "the move is used");
}

/// AS3: under an intent an after-move (one move block) may follow the attacks and is the round's one standard move;
/// without an intent it is refused like any late move.
#[test]
fn an_after_move_follows_the_attacks() {
    let mut s = session(timeline());
    let w = &mut s.w;
    place(w, HERO, V2::new(4, 4));
    place(w, FOE, V2::new(5, 4));
    let mv = sid(w, "move");
    let cut = sid(w, "t_cut");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(cut, FOE), false), "");
    assert!(!plan.add_step(w, HERO, PlanStep::after_move(mv, V2::new(3, 4)), false).is_empty(), "no intent: refused");
    intent(w, HERO, RoundIntent::Assault, RoundTarget::Unit(FOE));
    assert_eq!(plan.add_step(w, HERO, PlanStep::after_move(mv, V2::new(3, 4)), false), "");
    let last = plan.entries_for(w, HERO).pop().unwrap();
    assert_eq!((skill::kind(&w.cat.skills[last.skill]), last.pos_after), (Kind::Move, V2::new(3, 4)));
    assert!(plan.add_step(w, HERO, PlanStep::after_move(mv, V2::new(3, 3)), false).contains("already moved"), "one standard move a round");
    let mut again = Plan::new();
    assert_eq!(again.add_step(w, HERO, PlanStep::mv(mv, V2::new(0, -1), 1), false), "");
    assert!(again.add_step(w, HERO, PlanStep::after_move(mv, V2::new(4, 2)), false).contains("already moved"), "after a plain move");
    let mut far = Plan::new();
    assert_eq!(far.add_step(w, HERO, PlanStep::engage(cut, FOE), false), "");
    assert!(!far.add_step(w, HERO, PlanStep::after_move(mv, V2::new(1, 4)), false).is_empty(), "one move block only");
}

/// GD5: a stealthed unit that moved in block 1 is lost by a locked attack landing in block 3 (nothing dealt, AP spent,
/// no chase); an area on its tile still hits; when it did not move the attack hits.
#[test]
fn stealth_hides_a_unit_that_moved() {
    for (moves, area, found) in [(true, false, false), (true, true, true), (false, false, true)] {
        let mut s = session(timeline());
        let w = &mut s.w;
        place(w, HERO, V2::new(4, 3));
        place(w, FOE, V2::new(5, 3));
        status::grant_now(&mut w.units[FOE], "stealth", 2, 0, "test");
        intent(w, HERO, RoundIntent::Assault, RoundTarget::Unit(FOE));
        intent(w, FOE, RoundIntent::Guard, RoundTarget::Myself);
        let mut plan = Plan::new();
        if moves {
            assert_eq!(plan.add_step(w, FOE, PlanStep::mv(sid(w, "move"), V2::new(0, 1), 1), false), "");
        }
        for _ in 0..2 {
            assert_eq!(plan.add_step(w, HERO, PlanStep::self_skill(sid(w, "rest")), false), "");
        }
        let step = if area { PlanStep::area_at(sid(w, "t_burst"), V2::new(5, 4)) } else { PlanStep::engage(sid(w, "t_cut"), FOE) };
        if area {
            round_intent::set_target(w, HERO, RoundTarget::Cell(V2::new(5, 4))).unwrap();
            place(w, HERO, V2::new(4, 4));
        }
        assert_eq!(plan.add_step(w, HERO, step, false), "", "moves {moves}, area {area}");
        let attack_ap = plan.entries_for(w, HERO).last().unwrap().total_ap();
        let mut res = commit(w, &plan);
        let (hp, ap, hero_at) = (w.units[FOE].hp, w.units[HERO].ap, w.pos(HERO));
        run(w, &mut res, 1, 3);
        let case = format!("moves {moves}, area {area}");
        assert_eq!(w.units[FOE].hp < hp, found, "{case}: {:?}", res.log);
        assert_eq!(w.units[HERO].ap, ap - attack_ap, "{case}: AP spent");
        if !found {
            assert!(res.log.iter().any(|l| l.starts_with("[Lost]")), "{case}: {:?}", res.log);
            assert_eq!(w.pos(HERO), hero_at, "{case}: no chase toward it");
        }
    }
}

/// TQ23 (a): a unit's `default_intent` is its intent at battle start under the round intent; none in classic. Narrowed
/// by RULESET1-AI (AI1): an enemy re-picks at its first planning, so the default is read on a party member.
#[test]
fn default_intent_starts_the_battle() {
    let s = session(timeline());
    assert_eq!(round_intent::intent_of(&s.w, ALLY), Some(RoundIntent::Guard));
    assert!(round_intent::intent_of(&s.w, FOE_B).is_some(), "the enemy AI picked one");
    assert_eq!(round_intent::intent_of(&s.w, HERO), None, "none by default");
    let c = session(Ruleset::classic());
    assert_eq!(round_intent::intent_of(&c.w, FOE_B), None);
    assert!(c.w.round_states.is_empty(), "classic keeps no round state");
}
