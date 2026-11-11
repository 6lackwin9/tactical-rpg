//! PK2-S1 (`docs/sessions/PK2_GRAFTS.md` section 2, PK1 3.5 and 3.7, `FB-174` (5), (6)) in battle, on a scratch catalog
//! built here (no shipped data). Expectations come from the rules: a mark whose id is a unit track row's `mark`
//! moves the track at once on a unit holding tracks (each stage reached grants its passive for the rest of the battle;
//! the departure stage turns a party-side unit on everyone, `hostile_all`), never on a unit without tracks (it keeps
//! only the mark); the in-battle stage is the settle's formula (battle-start stage plus the current marks, held within
//! the row), so an `unmark` moves it back and a later mark counts from there; an `unmark` that takes a track-turned
//! unit below the departure stage calms it (clearing its end hostility), one that leaves it there does not, and a mark
//! back to the stage before the calm cancels it; the member grant `pierce_melee`
//! makes a melee hit skip armour under the rule pierce (the estimate too) and never a ranged one; a SUPPORT skill's
//! `unmark` removes the target's latest mark, and `uses_per_battle` refuses a further use in the planner and fails a
//! further start in the resolver; a use counts only once its costs are paid (a command failing unpaid keeps it).

use indexmap::IndexMap;
use rpgkit_battle::command::{self, Command};
use rpgkit_battle::encounter::HOSTILE_ALL;
use rpgkit_battle::grid::V2;
use rpgkit_battle::mark;
use rpgkit_battle::plan::{self, Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::session::{EncounterDef, Roster, Session};
use rpgkit_battle::trigger;
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, UnitDef};
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::ruleset::{Ruleset, PIERCE, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::Overrides;
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const HP: i32 = 500;
/// The foe's armour (the pierce tests need some to skip).
const FOE_ARMOR: i32 = 40;
/// The test track: mark `MARK`, three stages, the last a departure.
const TRACK: &str = "t_track";
const MARK: &str = "t_mark";
/// Stage one's resist (the passive's number the tests read back).
const RESIST: i32 = 7;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "t_jab", "name": "Jab", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY"}),
        json!({"id": "t_burst", "name": "Burst", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "reach_min": 1, "reach_max": 2, "area": "BURST"}),
        json!({"id": "t_inoculate", "name": "Inoculate", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 1, "target_mode": "ALLY", "reach_min": 0, "reach_max": 3,
               "unmark": {"mark": MARK}, "uses_per_battle": 1}),
    ];
    let passives = vec![
        json!({"id": "t_stage_one", "name": "Stage One", "effects": [{"type": "resist", "status": "bleed", "value": RESIST}], "slot": ""}),
        json!({"id": "t_stage_two", "name": "Stage Two", "effects": [{"type": "forbid_hands", "hands": 2}, {"type": "pierce_melee"}], "slot": ""}),
    ];
    let tracks = vec![json!({"id": TRACK, "name": "Track", "mark": MARK, "cause": "t_cause", "stages": [
        {"name": "One", "passive": "t_stage_one"}, {"name": "Two", "passive": "t_stage_two"}, {"name": "Three", "departure": "lost"}]})];
    let unit = |id: &str, side: &str, speed: i32, armor: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 100, "armor": armor, "speed": speed, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "passives": passives,
        "unit_tracks": tracks,
        "units": [unit("p_hero", "party", 14, 0), unit("p_ally", "party", 12, 0), unit("p_foe", "enemy", 10, FOE_ARMOR)],
    }))
}

struct Plain;

impl Roster for Plain {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, _r: &str) -> Overrides {
        Overrides::default()
    }
}

/// Hero (4,4), the ally below it (4,5), the foe beside the hero (5,4); the hero holds tracks (a member's battle unit)
/// at `stage` on the test track.
fn session(rules: &[&str], stage: i32) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    let rules: Vec<&str> = [TRIGGERS].into_iter().chain(rules.iter().copied()).collect();
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap());
    let mut s = Session::start(Arc::new(catalog()), def, &Plain);
    for (c, cell) in [(HERO, V2::new(4, 4)), (ALLY, V2::new(4, 5)), (FOE, V2::new(5, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    let mut tracks = IndexMap::new();
    if stage > 0 {
        tracks.insert(TRACK.to_string(), stage);
    }
    s.w.units[HERO].tracks = Some(tracks);
    s
}

/// Gives `c` a passive trigger row (source `passive:<id>`, limit none).
fn give(w: &mut World, c: UnitId, id: &str, on: &str, effects: Value) {
    let cat = w.cat.clone();
    let e = json!({"type": "trigger", "on": on, "when": {}, "effects": effects, "limit": "none"});
    w.units[c].apply_effects(&cat, &[e], &format!("passive:{id}"), false);
}

/// One round end of `w`'s triggers in a fresh resolver; its log.
fn round_end(w: &mut World) -> Vec<String> {
    let mut res = Resolver::new(BLOCKS, 1);
    trigger::round_end(&mut res, w);
    res.log
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

fn stage(w: &World, c: UnitId) -> Option<i32> {
    w.units[c].tracks.as_ref().map(|t| t.get(TRACK).copied().unwrap_or(0))
}

/// A track mark advances the hero's stage at once: stage one's passive is held now (its resist), the log names the
/// stage; a second mark reaches stage two (both member grants), each passive applied once.
#[test]
fn a_track_mark_takes_hold_at_once() {
    let mut w = session(&[], 0).w;
    give(&mut w, HERO, "breach", "round_end", json!([{"type": "mark", "mark": MARK, "value": 1}]));
    let log = round_end(&mut w);
    assert_eq!(stage(&w, HERO), Some(1));
    assert!(w.units[HERO].passives.contains(&"t_stage_one".to_string()));
    assert_eq!(w.units[HERO].status_resist.get("bleed").copied(), Some(RESIST), "stage one's resist");
    assert!(log.iter().any(|l| l.ends_with("p_hero is marked t_mark +1; p_hero's Track is at stage 1 (One)")), "{log:?}");
    assert!(!w.units[HERO].pierce_melee && w.units[HERO].forbid_hands == 0);
    round_end(&mut w);
    assert_eq!(stage(&w, HERO), Some(2));
    assert!(w.units[HERO].pierce_melee && w.units[HERO].forbid_hands == 2, "stage two's member grants");
    assert_eq!(w.units[HERO].passives.iter().filter(|p| p.starts_with("t_stage")).count(), 2, "each passive once");
    assert_eq!(w.units[HERO].max_hp, HP, "the battle's max HP stays");
    assert_eq!(w.marks.of(HERO), vec![(MARK.to_string(), 1), (MARK.to_string(), 1)], "the marks are recorded as before");
}

/// A value past the stages stops at the top; a unit without tracks (the foe) keeps only its mark; another mark id moves
/// nothing.
#[test]
fn only_a_track_holder_moves_and_only_by_its_mark() {
    let mut w = session(&[], 0).w;
    give(&mut w, FOE, "breach", "round_end", json!([{"type": "mark", "mark": MARK, "value": 1}]));
    give(&mut w, HERO, "scar", "round_end", json!([{"type": "mark", "mark": "other", "value": 1}]));
    round_end(&mut w);
    assert_eq!(stage(&w, FOE), None, "no tracks");
    assert!(!w.units[FOE].passives.contains(&"t_stage_one".to_string()));
    assert_eq!(w.marks.of(FOE), vec![(MARK.to_string(), 1)]);
    assert_eq!(stage(&w, HERO), Some(0), "another mark id");
    w.marks.record(HERO, MARK, 50);
    assert_eq!(mark::sync_tracks(&mut w, HERO, MARK), vec!["p_hero's Track is at stage 3 (Three)".to_string(), "p_hero turns on everyone".to_string()]);
    assert!(mark::sync_tracks(&mut w, HERO, MARK).is_empty(), "nothing moved: no parts");
    assert_eq!(stage(&w, HERO), Some(3), "held at the top");
}

/// The departure stage turns the party-side hero on everyone at once; at the end it is hostile by the track; an
/// `unmark` that calms it clears that (it returns next round).
#[test]
fn the_departure_stage_turns_the_unit_and_an_unmark_calms_it() {
    let mut w = session(&[], 2).w;
    give(&mut w, HERO, "breach", "round_end", json!([{"type": "mark", "mark": MARK, "value": 1}]));
    let log = round_end(&mut w);
    assert!(w.sides.is_rogue(HERO) && !w.allies.contains(&HERO), "hostile to all");
    assert!(log.iter().any(|l| l.ends_with("p_hero's Track is at stage 3 (Three); p_hero turns on everyone")), "{log:?}");
    assert_eq!(mark::hostile_at_end(&w, HERO), TRACK);
    assert_eq!(w.marks.start_party(), &[HERO, ALLY], "the start side keeps the hero for the settle's pairing");
    assert_eq!(mark::unmark(&mut w, HERO, MARK, "Salt"), "p_hero's t_mark undone; p_hero's Track is at stage 2 (Two); p_hero calms at the next round start");
    assert_eq!(stage(&w, HERO), Some(2), "the unmark moves the stage back");
    assert_eq!(mark::hostile_at_end(&w, HERO), "", "calming: not hostile at the end");
    // a unit already turned (by a phase) is not turned again, and only a track's departure is a hostile end
    let mut w = session(&[], 0).w;
    assert!(w.set_side(HERO, HOSTILE_ALL));
    assert_eq!(mark::hostile_at_end(&w, HERO), "");
}

/// `c` gets a mark of the test track's id (`value`) as the trigger effect records it, the tracks re-read; the parts.
fn breach(w: &mut World, c: UnitId, value: i32) -> Vec<String> {
    w.marks.record(c, MARK, value);
    mark::sync_tracks(w, c, MARK)
}

/// The settle's stage from the battle's marks: the start stage plus the current marks, held within the row.
fn settle_stage(w: &World, c: UnitId, start: i32) -> i32 {
    w.cat.unit_tracks[TRACK].clamp(i64::from(start) + w.marks.sum_of(c, MARK))
}

/// Breach, unmark, breach (review M2): at the stage before the departure, a breach turns the hero; the unmark moves it
/// back and calms it, it rejoins at the round start; a second breach counts from the start stage again, so it reaches
/// the departure stage and turns the hero once more, as the settle's formula reads the marks. Each stage passive is
/// held once.
#[test]
fn breach_unmark_breach_matches_the_settle() {
    let start = 2;
    let mut w = session(&[], start).w;
    assert!(breach(&mut w, HERO, 1).iter().any(|p| p == "p_hero turns on everyone"));
    assert!(mark::unmark(&mut w, HERO, MARK, "Salt").ends_with("p_hero calms at the next round start"));
    assert_eq!((stage(&w, HERO), settle_stage(&w, HERO, start)), (Some(2), 2));
    assert_eq!(mark::round_start(&mut w), vec![(HERO, "Salt".to_string())], "rejoins at the round start");
    assert!(!w.sides.is_rogue(HERO) && w.allies.contains(&HERO));
    assert_eq!(mark::hostile_at_end(&w, HERO), "");
    assert_eq!(breach(&mut w, HERO, 1), vec!["p_hero's Track is at stage 3 (Three)".to_string(), "p_hero turns on everyone".to_string()]);
    assert_eq!((stage(&w, HERO), settle_stage(&w, HERO, start)), (Some(3), 3), "the battle and the settle agree");
    assert!(w.sides.is_rogue(HERO));
    assert_eq!(mark::hostile_at_end(&w, HERO), TRACK);
    assert_eq!(w.units[HERO].passives.iter().filter(|p| p.starts_with("t_stage")).count(), 0, "stage 2 at the start: its passives come from the build, not the battle");

    // the second breach lands before the round start: the calm is cancelled, the hero stays turned
    let mut w = session(&[], start).w;
    breach(&mut w, HERO, 1);
    mark::unmark(&mut w, HERO, MARK, "Salt");
    assert!(w.marks.calming(HERO));
    assert_eq!(breach(&mut w, HERO, 1), vec!["p_hero's Track is at stage 3 (Three)".to_string(), "p_hero stays turned on everyone".to_string()]);
    assert!(!w.marks.calming(HERO) && mark::round_start(&mut w).is_empty() && w.sides.is_rogue(HERO), "no calm");
    assert_eq!(mark::hostile_at_end(&w, HERO), TRACK);

    // an overshoot: two breaches held at the top; an unmark leaves the hero at the departure stage, so no calm
    let mut w = session(&[], start).w;
    breach(&mut w, HERO, 1);
    assert!(breach(&mut w, HERO, 1).is_empty(), "held at the top");
    assert_eq!(mark::unmark(&mut w, HERO, MARK, "Salt"), "p_hero's t_mark undone");
    assert!(!w.marks.calming(HERO) && settle_stage(&w, HERO, start) == 3);
    assert_eq!(mark::hostile_at_end(&w, HERO), TRACK);

    // from stage 0: up, back down and up again grants each passive once
    let mut w = session(&[], 0).w;
    breach(&mut w, HERO, 2);
    mark::unmark(&mut w, HERO, MARK, "Salt");
    assert_eq!(stage(&w, HERO), Some(0));
    breach(&mut w, HERO, 2);
    assert_eq!(stage(&w, HERO), Some(2));
    assert_eq!(w.units[HERO].passives.iter().filter(|p| p.starts_with("t_stage")).count(), 2, "each passive once");
}

/// `pierce_melee` skips armour on a melee hit under the rule pierce (the hit equals the armour-free formula, the
/// estimate and the command's forecast too, PR5-26); without the rule, without the grant or on a ranged hit the armour counts.
#[test]
fn pierce_melee_skips_armour_on_melee_hits() {
    for (rules, grant, pierces) in [(vec![PIERCE], true, true), (vec![PIERCE], false, false), (vec![], true, false)] {
        let mut w = session(&rules, 0).w;
        w.units[HERO].pierce_melee = grant;
        let s = &w.cat.skills[sid(&w, "t_jab")];
        let input = |ignore_armor: bool| HitInput { ranged: false, skill_mult: w.units[HERO].power_for(s), ignore_armor, ..HitInput::default() };
        let (bare, armoured) = (damage::compute(&w.units[HERO], &w.units[FOE], input(true)).final_dmg, damage::compute(&w.units[HERO], &w.units[FOE], input(false)).final_dmg);
        assert!(bare > armoured, "the foe's armour matters");
        let est = plan::expected_with(&w, HERO, FOE, sid(&w, "t_jab"), None, 0.0, 1.0, false);
        assert_eq!(est, if pierces { bare } else { armoured }, "estimate: rules {rules:?}, grant {grant}");
        assert_eq!(command::pierces_melee(&w, HERO, false), pierces);
        assert!(!command::pierces_melee(&w, HERO, true), "never a ranged hit");
        let before = w.units[FOE].hp;
        let mut plan = Plan::new();
        let step: Step = PlanStep::engage(sid(&w, "t_jab"), FOE);
        assert_eq!(plan.add_step(&w, HERO, step, true), "");
        let id = sid(&w, "t_jab");
        let cmd = plan.entries_for(&w, HERO).into_iter().find(|e| e.skill == id).unwrap().build_command(&w, HERO);
        // PR5-26 (audit B5): the command's forecast ("Armour pierced") reads the grant on a melee attack too, and
        // (review F4) on a melee unit's area attack, as `land` pierces its hits
        assert_eq!(cmd.pierces(&w), pierces, "forecast: rules {rules:?}, grant {grant}");
        let mut area = Plan::new();
        assert_eq!(area.add_step(&w, HERO, PlanStep::area_at(sid(&w, "t_burst"), V2::new(5, 4)), true), "");
        let burst = sid(&w, "t_burst");
        let blast = area.entries_for(&w, HERO).into_iter().find(|e| e.skill == burst).unwrap().build_command(&w, HERO);
        assert!(matches!(blast.kind, command::CmdKind::Area { .. }));
        assert_eq!(blast.pierces(&w), pierces, "area forecast: rules {rules:?}, grant {grant}");
        let mut res = Resolver::new(BLOCKS, 1);
        assert!(res.schedule(HERO, cmd, 1));
        run(&mut w, &mut res);
        let dealt = before - w.units[FOE].hp;
        assert!(dealt > 0);
        if pierces {
            assert!(dealt >= bare.min(est) && dealt > armoured, "a pierced hit ({dealt}) beats the armoured one ({armoured})");
        } else {
            assert!(dealt < bare, "an armoured hit ({dealt}) is below the bare one ({bare})");
        }
    }
}

fn run(w: &mut World, res: &mut Resolver) {
    for b in 1..=BLOCKS {
        for i in res.prepare_block(w, b) {
            res.execute_single(w, i);
        }
    }
}

/// `c` casts Inoculate on `t` alone in a fresh resolver; the command after the round.
fn inoculate(w: &mut World, c: UnitId, t: UnitId) -> Command {
    let mut plan = Plan::new();
    let id = sid(w, "t_inoculate");
    assert_eq!(plan.add_step(w, c, PlanStep::ally(id, t), true), "", "planned");
    let cmd = plan.entries_for(w, c).into_iter().find(|e| e.skill == id).expect("the entry").build_command(w, c);
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(c, cmd, 1), "booked");
    run(w, &mut res);
    res.scheduled.pop().unwrap().command
}

/// The skill's `unmark` takes the target's latest mark back (its log says so); its `uses_per_battle` 1 refuses a second
/// use in the planner (and a second planned in one plan); a second start fails at the resolver.
#[test]
fn the_skill_unmark_is_once_a_battle() {
    let mut w = session(&[], 0).w;
    let s = w.cat.skills[sid(&w, "t_inoculate")].clone();
    assert_eq!((s.unmark.as_str(), s.uses_per_battle), (MARK, 1), "the fields load");
    w.marks.record(HERO, MARK, 1);
    w.marks.record(HERO, MARK, 1);
    let cmd = inoculate(&mut w, ALLY, HERO);
    assert!(cmd.log.contains("p_hero's t_mark undone"), "{}", cmd.log);
    assert_eq!(w.marks.of(HERO).len(), 1, "the latest went");
    assert_eq!(mark::uses_of(&w, ALLY, "t_inoculate"), 1);
    assert_eq!(mark::uses_problem(&w, ALLY, &s, 0), "Inoculate is used once a battle.");
    let mut plan = Plan::new();
    let step: Step = PlanStep::ally(sid(&w, "t_inoculate"), HERO);
    assert_eq!(plan.add_step(&w, ALLY, step, true), "Inoculate is used once a battle.", "the planner refuses");
    // a fresh battle: one planned, the second in the same plan is refused
    let mut w = session(&[], 0).w;
    let id = sid(&w, "t_inoculate");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, ALLY, PlanStep::ally(id, HERO), true), "");
    assert_eq!(plan.add_step(&w, ALLY, PlanStep::ally(id, HERO), true), "Inoculate is used once a battle.");
    // the resolver: a command booked past the limit fails at its start
    let entry = plan.entries_for(&w, ALLY).into_iter().find(|e| e.skill == id).unwrap();
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(ALLY, entry.build_command(&w, ALLY), 1));
    assert!(res.schedule(ALLY, entry.build_command(&w, ALLY), 3));
    run(&mut w, &mut res);
    assert!(res.log.iter().any(|l| l == "[Action Failed] p_ally's Inoculate is used up this battle."), "{:?}", res.log);
    assert_eq!(mark::uses_of(&w, ALLY, "t_inoculate"), 1);
    // with nothing to undo, the use is spent with a line
    let mut w = session(&[], 0).w;
    let cmd = inoculate(&mut w, ALLY, HERO);
    assert!(cmd.log.contains("no t_mark to undo"), "{}", cmd.log);
}

/// Review L2: a use counts only once the command's costs are paid; one failing unpaid (a meter gate it cannot meet,
/// rule unit_meters) keeps the use, so the next start goes through and counts it.
#[test]
fn a_command_failing_unpaid_keeps_its_use() {
    let mut w = session(&[UNIT_METERS], 0).w;
    w.marks.record(HERO, MARK, 1);
    w.marks.record(HERO, MARK, 1);
    let id = sid(&w, "t_inoculate");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, ALLY, PlanStep::ally(id, HERO), true), "");
    let entry = plan.entries_for(&w, ALLY).into_iter().find(|e| e.skill == id).unwrap();
    let mut gated = entry.build_command(&w, ALLY);
    gated.gate = Some(rpgkit_data::MeterGate { meter: "t_strain".into(), min: 5 });
    let mut res = Resolver::new(BLOCKS, 1);
    assert!(res.schedule(ALLY, gated, 1));
    assert!(res.schedule(ALLY, entry.build_command(&w, ALLY), 3));
    run(&mut w, &mut res);
    assert!(res.log.iter().any(|l| l.contains("fails unpaid")), "{:?}", res.log);
    assert!(!res.log.iter().any(|l| l.contains("used up")), "{:?}", res.log);
    assert_eq!(mark::uses_of(&w, ALLY, "t_inoculate"), 1, "only the paid start counts");
    assert_eq!(w.marks.of(HERO).len(), 1, "the paid start took one mark back");
}
