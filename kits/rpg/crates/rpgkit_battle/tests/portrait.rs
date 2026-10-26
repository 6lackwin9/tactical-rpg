//! Portrait cue evaluation (PORTRAITFX1-S, docs/specs/PORTRAITFX1.md 5.2-5.3; checks S3-S5 of PORTRAITFX1_TASKS.md
//! 1.2) on rows and a scratch catalog built here: which state rows hold for a look on a surface and under a ruleset,
//! which event rows a step's events fire and in what order, and the events of one step read from the HP before and
//! after and the step's log lines (the resolver's own line formats), without touching the session.

use indexmap::IndexMap;
use rpgkit_battle::portrait::{holds, matches, reactions, session_step_events, step_events, Look, StepUnit};
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
