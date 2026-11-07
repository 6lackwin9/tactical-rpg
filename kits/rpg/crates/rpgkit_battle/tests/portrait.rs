//! Portrait cue evaluation (PORTRAITFX1-S, docs/specs/PORTRAITFX1.md 5.2-5.3; checks S3-S5 of PORTRAITFX1_TASKS.md
//! 1.2) on rows and a scratch catalog built here: which state rows hold for a look on a surface and under a ruleset,
//! which event rows a step's events fire and in what order, and the events of one step read from the HP before and
//! after and the step's log lines (the resolver's own line formats), without touching the session. PORTRAITFX2 5.3:
//! who gets each battle event and when (`battle_start`, `ambush`, `round_start`, `skill` with `min_level`,
//! `ally_down`, `victory`) and the `unspent` state.

use indexmap::IndexMap;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::portrait::{ally_down_events, holds, matches, reactions, reactions_at, round_events, session_step_events, skill_event, start_events, step_events, victory_events, Look, StepUnit, ALLY_DOWN, AMBUSH, AMBUSH_DEPLOYMENT, BATTLE_START, ROUND_START, SKILL, VICTORY};
use rpgkit_battle::resolver::ScheduledAction;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_data::{Catalog, PortraitCue};
use rpgkit_rules::unit::Stance;
use serde_json::{json, Value};
use std::sync::Arc;

fn cue(v: Value) -> PortraitCue {
    PortraitCue::from_obj(v.as_object().unwrap())
}

fn ids(rows: &[&PortraitCue]) -> Vec<String> {
    rows.iter().map(|r| r.id.clone()).collect()
}

fn look(hp: i32, max_hp: i32) -> Look {
    Look { hp, max_hp, ruleset: "classic".into(), ..Look::default() }
}

#[test]
fn state_rows_hold_while_their_match_is_true() {
    let status = cue(json!({"id": "marked", "trigger": "status", "status": "stun", "effect": "fade"}));
    let mut l = look(10, 10);
    assert!(!matches(&status, &l));
    l.statuses.push("stun".into());
    assert!(matches(&status, &l), "only with the status");
    let wounded = cue(json!({"id": "wounded", "trigger": "condition", "condition": "wounded", "effect": "desaturate"}));
    assert!(!matches(&wounded, &l));
    l.conditions.push("wounded".into());
    assert!(matches(&wounded, &l));
    let half = cue(json!({"id": "hp_half", "trigger": "hp", "below": 50, "effect": "desaturate"}));
    assert!(matches(&half, &look(49, 100)), "49 of 100 is below 50 %");
    assert!(!matches(&half, &look(50, 100)), "50 of 100 is not");
    assert!(!matches(&half, &look(0, 0)), "no max HP: no hp row matches");
    let down = cue(json!({"id": "downed", "trigger": "down", "effect": "still"}));
    assert!(!matches(&down, &look(0, 0)));
    assert!(matches(&down, &Look { down: true, ..look(0, 10) }));
    let shaken = cue(json!({"id": "shaken", "trigger": "derived", "stat": "nerve", "drop_pct": 25, "effect": "tint"}));
    let with = |with: i32, without: i32| Look { derived: vec![("nerve".into(), with, without)], ..look(10, 10) };
    assert!(matches(&shaken, &with(75, 100)), "a drop of 25 % matches");
    assert!(!matches(&shaken, &with(76, 100)), "24 % does not");
    assert!(!matches(&shaken, &with(0, 0)), "nothing to drop from");
    assert!(!matches(&shaken, &look(10, 10)), "a look without the stat");
    let low = cue(json!({"id": "nerve_low", "trigger": "derived", "stat": "nerve", "below": 20, "effect": "tint"}));
    assert!(matches(&low, &with(19, 19)));
    assert!(!matches(&low, &with(20, 20)));
    let event = cue(json!({"id": "hit_flash", "event": "hit", "ms": 140}));
    assert!(!matches(&event, &Look { down: true, ..l.clone() }), "an event row never holds");
}

#[test]
fn holds_keeps_the_rows_of_the_surface_and_the_ruleset() {
    let rows = vec![
        cue(json!({"id": "hp_half", "trigger": "hp", "below": 50, "effect": "desaturate"})),
        cue(json!({"id": "hp_panel", "trigger": "hp", "below": 50, "effect": "tint", "surfaces": ["panel"]})),
        cue(json!({"id": "hp_timeline", "trigger": "hp", "below": 50, "effect": "fade", "rulesets": ["timeline"]})),
        cue(json!({"id": "hit_flash", "event": "hit", "ms": 140})),
        cue(json!({"id": "downed", "trigger": "down", "effect": "still"})),
    ];
    let hurt = look(10, 100);
    assert_eq!(ids(&holds(&rows, &hurt, "dialogue")), ["hp_half"], "a panel row is left out on dialogue; a timeline row under classic");
    assert_eq!(ids(&holds(&rows, &hurt, "panel")), ["hp_half", "hp_panel"]);
    let timeline = Look { ruleset: "timeline".into(), ..hurt.clone() };
    assert_eq!(ids(&holds(&rows, &timeline, "panel")), ["hp_half", "hp_panel", "hp_timeline"], "row order");
    assert!(holds(&rows, &look(100, 100), "panel").is_empty());
}

#[test]
fn reactions_follow_the_events_then_the_rows() {
    let rows = vec![
        cue(json!({"id": "hit_flash", "event": "hit", "ms": 140})),
        cue(json!({"id": "crit_flash", "event": "crit", "ms": 180})),
        cue(json!({"id": "hit_jolt", "event": "hit", "effect": "jolt", "preset": "hit"})),
        cue(json!({"id": "crit_jolt", "event": "crit", "effect": "jolt", "preset": "crit", "surfaces": ["panel"]})),
        cue(json!({"id": "crit_timeline", "event": "crit", "effect": "fade", "ms": 300, "rulesets": ["timeline"]})),
        cue(json!({"id": "hp_half", "trigger": "hp", "below": 50, "effect": "desaturate"})),
    ];
    assert_eq!(ids(&reactions(&rows, &["crit", "hit"], "panel", "classic")), ["crit_flash", "crit_jolt", "hit_flash", "hit_jolt"]);
    assert_eq!(ids(&reactions(&rows, &["crit"], "dialogue", "timeline")), ["crit_flash", "crit_timeline"]);
    assert!(reactions(&rows, &["heal"], "panel", "classic").is_empty());
}

fn unit(name: &str, before: i32, after: i32, guarded: bool) -> StepUnit<'_> {
    StepUnit { name, hp_before: before, hp_after: after, guarded }
}

fn lines(l: &[&str]) -> Vec<String> {
    l.iter().map(|s| s.to_string()).collect()
}

#[test]
fn step_events_read_the_hp_and_the_lines() {
    let units = [unit("Rook", 20, 13, false), unit("Goblin", 20, 6, false), unit("Shield", 20, 17, true), unit("Bard", 10, 14, false), unit("Idle", 20, 20, false), unit("Brute", 30, 30, false)];
    // a single attack that crits, an area attack, a heal and a break, in the resolver's formats
    let step = lines(&[
        "[Attack] Rook struck Goblin for 14 CRIT (HP 6/20)",
        "[Attack] Goblin's Sweep hit 2 units for 10 damage (Rook 7, Shield 3)",
        "[Support] Bard mends Bard for 4",
        "[Break] Brute's Smash is broken",
    ]);
    assert_eq!(step_events(&units, &step), [(0, "hit"), (1, "crit"), (2, "guard_hit"), (3, "heal"), (5, "break")]);
    let area_crit = lines(&["[Attack] Goblin's Sweep hit 2 units for 22 damage (Shield 3, Rook 19 CRIT DEFEATED)"]);
    assert_eq!(step_events(&units[..1], &area_crit), [(0, "crit")], "an area part's crit");
    assert_eq!(step_events(&[unit("Rook", 20, 20, false)], &lines(&["[Break] Rook is broken"])), [(0, "break")]);
    assert_eq!(step_events(&[unit("Rook", 20, 10, false)], &lines(&["[Break] Rook is broken"])), [(0, "hit"), (0, "break")], "a hit that breaks");
    assert_eq!(step_events(&[unit("Rook", 20, 10, true)], &lines(&["[Attack] Mage struck Rook for 10 CRIT (HP 10/20)"])), [(0, "crit")], "a crit wins over the guard");
    assert_eq!(step_events(&[unit("Rook", 20, 10, false)], &lines(&["[Attack] Rookie struck Goblin for 9 CRIT (HP 1/20)"])), [(0, "hit")], "another unit's crit");
    assert!(step_events(&[unit("Idle", 5, 5, true)], &step).is_empty(), "a unit untouched gets nothing");
}

/// A two-unit fight on a scratch catalog (a strike, a guard and the engine skills).
fn session() -> Session {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 40, "speed": 10, "attack_range": 1, "skill_cap": 5});
    let cat = Catalog::from_value(&json!({"skills": skills, "units": [unit("pf_hero", "party"), unit("pf_foe", "enemy")]}));
    let row = json!({"id": "pf_fight", "name": "Fight", "party": ["pf_hero"], "enemies": ["pf_foe"], "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 4, "rng_seed": 7});
    Session::start(Arc::new(cat), EncounterDef::from_obj(row.as_object().unwrap()), &IndexMap::new())
}

/// One unit's name, HP, guard and stance.
type UnitSeen = (String, i32, f64, Stance);

/// What the call could change: each unit's name, HP, guard and stance, and the log lengths.
fn snapshot(s: &Session) -> (Vec<UnitSeen>, usize, usize) {
    (s.w.units.iter().map(|u| (u.name.clone(), u.hp, u.guard, u.stance)).collect(), s.log.len(), s.resolver.log.len())
}

#[test]
fn session_step_events_read_the_session_only() {
    let mut s = session();
    assert_eq!(s.w.units.len(), 2);
    let before: Vec<i32> = s.w.units.iter().map(|u| u.hp).collect();
    // the step as it left the units: the hero struck with a crit, the foe guarding and hit
    s.w.units[0].hp -= 9;
    s.w.units[1].hp -= 3;
    s.w.units[1].guard = 0.3;
    let (hero, foe) = (s.w.units[0].name.clone(), s.w.units[1].name.clone());
    let step = vec![format!("[Attack] {foe} struck {hero} for 9 CRIT (HP 31/40)"), format!("[Attack] {hero} struck {foe} for 3 (HP 37/40)")];
    let seen = snapshot(&s);
    assert_eq!(session_step_events(&s, &before, &step), [(0, "crit"), (1, "guard_hit")]);
    assert_eq!(snapshot(&s), seen, "the session is the same after the call");
    s.w.units[1].guard = 0.0;
    s.w.units[1].stance = Stance::Defensive;
    assert_eq!(session_step_events(&s, &before, &step), [(0, "crit"), (1, "guard_hit")], "a defensive stance guards too");
    s.w.units[1].stance = Stance::default();
    assert_eq!(session_step_events(&s, &before, &step)[1], (1, "hit"));
    assert_eq!(session_step_events(&s, &before[..1], &step), [(0, "crit")], "units past hp_before are left out");
}

// ---- PORTRAITFX2 5.3: the battle events and the `unspent` state ----------------------------------------------------

/// A two-against-two fight on the scratch catalog of `session` plus a level-3 focus skill, with `deployment`.
fn fight(deployment: &str) -> Session {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "rally", "name": "Rally", "kind": "FOCUS", "level": 3, "blocks": 1}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 40, "speed": 10, "attack_range": 1, "skill_cap": 5});
    let units = [unit("pf_hero", "party"), unit("pf_ally", "party"), unit("pf_foe", "enemy"), unit("pf_foe2", "enemy")];
    let cat = Catalog::from_value(&json!({"skills": skills, "units": units}));
    let row = json!({"id": "pf_fight", "name": "Fight", "party": ["pf_hero", "pf_ally"], "enemies": ["pf_foe", "pf_foe2"], "deployment": deployment, "total_blocks": 6, "unit_ap": 10, "ap_regen": 0, "max_rounds": 4, "rng_seed": 7});
    let s = Session::start(Arc::new(cat), EncounterDef::from_obj(row.as_object().unwrap()), &IndexMap::new());
    assert_eq!((s.w.units.len(), s.w.allies.len()), (4, 2), "fixture: two a side");
    s
}

/// The player side's and the enemy side's unit indices.
fn sides(s: &Session) -> (Vec<usize>, Vec<usize>) {
    (0..s.w.units.len()).partition(|i| s.w.allies.contains(i))
}

fn each(units: &[usize], event: &'static str) -> Vec<(usize, &'static str)> {
    units.iter().map(|&i| (i, event)).collect()
}

#[test]
fn battle_start_goes_to_every_living_unit_and_ambush_to_the_player_side_in_an_ambush_only() {
    let mut s = fight("standard");
    let all: Vec<usize> = (0..s.w.units.len()).collect();
    assert_eq!(start_events(&s), each(&all, BATTLE_START), "no ambush in a standard deployment");
    let mut s2 = fight(AMBUSH_DEPLOYMENT);
    let (party, _) = sides(&s2);
    assert_eq!(start_events(&s2), [each(&all, BATTLE_START), each(&party, AMBUSH)].concat(), "the ambush goes to the player side");
    // a unit down at the start gets neither
    let (fallen, rest) = (party[0], party[1]);
    s2.w.units[fallen].hp = 0;
    let living: Vec<usize> = all.iter().copied().filter(|&i| i != fallen).collect();
    assert_eq!(start_events(&s2), [each(&living, BATTLE_START), each(&[rest], AMBUSH)].concat());
    s.def.deployment = "centre".into();
    assert!(!start_events(&s).iter().any(|e| e.1 == AMBUSH), "only the ambush deployment");
}

#[test]
fn round_start_fires_when_the_round_rises_and_never_under_the_round_intent_rule() {
    let mut s = fight("standard");
    let all: Vec<usize> = (0..s.w.units.len()).collect();
    let now = s.current_round;
    assert!(round_events(&s, now).is_empty(), "the same round");
    assert_eq!(round_events(&s, now - 1), each(&all, ROUND_START), "every living unit");
    s.w.units[all[3]].hp = 0;
    assert_eq!(round_events(&s, now - 1), each(&all[..3], ROUND_START), "not a downed unit");
    s.w.ruleset.rules.push(rpgkit_rules::ruleset::ROUND_INTENT);
    assert!(rpgkit_battle::round_intent::on(&s.w), "fixture: the rule is on");
    assert!(round_events(&s, now - 1).is_empty(), "decision Q7: no round_start under the timeline's round intent");
}

/// Schedules `kind` named `name` for unit `unit` and marks it as the last popped action of this round.
fn popped(s: &mut Session, unit: usize, name: &str, kind: CmdKind, cancelled: bool) {
    let mut command = Command::new(unit, 0, 1, kind);
    command.name = name.into();
    s.resolver.scheduled.push(ScheduledAction { unit, command, start: 1, impact: 1, end: 1, cancelled, landed: false, broken: false });
    s.last_popped = Some((s.current_round, 1, s.resolver.scheduled.len() - 1));
}

fn focus() -> CmdKind {
    CmdKind::Focus { boost: 0.2, rounds: 1 }
}

#[test]
fn skill_goes_to_the_actor_with_its_skill_level_and_rows_read_min_level() {
    let mut s = fight("standard");
    assert_eq!(skill_event(&s), None, "nothing popped yet");
    let level = |s: &Session, name: &str| s.w.cat.skills.iter().find(|k| k.name == name).map(|k| k.level).expect("a skill row");
    popped(&mut s, 1, "Rally", focus(), false);
    assert_eq!(skill_event(&s), Some((1, level(&s, "Rally"))), "the acting unit, at its skill's level");
    popped(&mut s, 1, "Guard", CmdKind::Defend { mitigation: 0.3, guard_rounds: 1, later_mitigation: 0.3, grant_status: String::new(), status_rounds: 0, extra_grants: Vec::new(), guard_broken: false }, false);
    assert_eq!(skill_event(&s), None, "a defend is not a skill command");
    popped(&mut s, 0, "Rally", focus(), true);
    assert_eq!(skill_event(&s), None, "a cancelled action");
    popped(&mut s, 0, "Rally", focus(), false);
    s.last_popped = s.last_popped.map(|(r, b, i)| (r - 1, b, i));
    assert_eq!(skill_event(&s), None, "a past round's action");
    // the rows: min_level 0 = any skill; above 0 the skill's level must reach it; `reactions` is level 0
    let rows = vec![cue(json!({"id": "any_skill", "event": "skill", "ms": 140})), cue(json!({"id": "big_skill", "event": "skill", "min_level": 3, "effect": "face", "expression": "angry", "ms": 1200}))];
    assert_eq!(ids(&reactions_at(&rows, &[SKILL], 2, "panel", "classic")), ["any_skill"]);
    assert_eq!(ids(&reactions_at(&rows, &[SKILL], 3, "panel", "classic")), ["any_skill", "big_skill"]);
    assert_eq!(ids(&reactions_at(&rows, &[SKILL], 4, "panel", "classic")), ["any_skill", "big_skill"]);
    assert_eq!(ids(&reactions(&rows, &[SKILL], "panel", "classic")), ["any_skill"]);
    let hit = vec![cue(json!({"id": "hit_flash", "event": "hit", "ms": 140}))];
    assert_eq!(ids(&reactions_at(&hit, &["hit"], 0, "panel", "classic")), ["hit_flash"], "other events do not read the level");
}

#[test]
fn ally_down_goes_to_the_other_living_units_of_the_fallen_side_once() {
    let mut s = fight("standard");
    let (party, foes) = sides(&s);
    let before: Vec<i32> = s.w.units.iter().map(|u| u.hp).collect();
    assert!(ally_down_events(&s, &before).is_empty(), "nobody fell");
    s.w.units[party[1]].hp = 0;
    assert_eq!(ally_down_events(&s, &before), each(&party[..1], ALLY_DOWN), "the fallen's side, not the fallen unit, not the foes");
    assert!(session_step_events(&s, &before, &[]).contains(&(party[0], ALLY_DOWN)), "with the step's events (at the impact)");
    // both foes fall in the same step: the hero still gets one, the side with nobody standing none
    s.w.units[foes[0]].hp = 0;
    s.w.units[foes[1]].hp = 0;
    assert_eq!(ally_down_events(&s, &before), each(&party[..1], ALLY_DOWN), "once per unit per step");
    // a unit down before the step does not fall in it
    let after: Vec<i32> = s.w.units.iter().map(|u| u.hp).collect();
    assert!(ally_down_events(&s, &after).is_empty(), "already down before the step");
    let mut s = fight("standard");
    let before: Vec<i32> = s.w.units.iter().map(|u| u.hp).collect();
    s.w.units[foes[0]].hp = 0;
    s.w.units[party[0]].hp = 0;
    assert_eq!(ally_down_events(&s, &before), [(party[1], ALLY_DOWN), (foes[1], ALLY_DOWN)], "both sides, index order");
}

#[test]
fn victory_goes_to_the_player_side_on_the_winning_step_only() {
    let mut s = fight("standard");
    let (party, _) = sides(&s);
    assert!(victory_events(&s, "").is_empty(), "the battle goes on");
    s.outcome = "won".into();
    assert_eq!(victory_events(&s, ""), each(&party, VICTORY));
    assert!(victory_events(&s, "won").is_empty(), "only the step that ended it");
    s.w.units[party[0]].hp = 0;
    assert_eq!(victory_events(&s, ""), each(&party[1..], VICTORY), "the living only");
    s.outcome = "lost".into();
    assert!(victory_events(&s, "").is_empty(), "not a loss");
}

#[test]
fn unspent_rows_hold_while_points_are_unspent() {
    let row = cue(json!({"id": "face_level_up", "trigger": "unspent", "effect": "face", "expression": "angry", "surfaces": ["roster"]}));
    let mut l = look(10, 10);
    assert!(!matches(&row, &l), "0 points");
    l.unspent = 2;
    assert!(matches(&row, &l));
    assert_eq!(ids(&holds(std::slice::from_ref(&row), &l, "roster")), ["face_level_up"]);
    assert!(holds(std::slice::from_ref(&row), &l, "panel").is_empty(), "the roster surfaces only");
}
