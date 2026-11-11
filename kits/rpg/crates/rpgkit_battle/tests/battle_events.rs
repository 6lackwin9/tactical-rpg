//! PR5-32 (`LAYER-UP-EVENTS`) on a scratch catalog built here (no shipped data): the resolver's typed events and the
//! planners' typed refusals. Every typed event names a line of `Resolver::log` that it describes (a hit is an `[Attack]`
//! line, its crit and zero flags what the line says, a break a `[Break]` line of that unit, ...), every line a screen
//! reacts to has its event, and the log's text is the same with or without anyone reading the events. A refusal's
//! fields give the planner's text back (`refusal::text`), the HP cost and a meter gate in fields, through the command
//! host's JSON too (`Reason::to_json` / `from_json`). The Field rows' area marks (`area_mark`) take the Assault row for
//! an entry locked on a target, else the round's intent.

use indexmap::IndexMap;
use rpgkit_battle::command;
use rpgkit_battle::event::{self, EventKind};
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanEntry, PlanStep};
use rpgkit_battle::refusal::{self, Reason};
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, HP_COST, POISE, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const HERO: UnitId = 0;
const FOE: UnitId = 2;
/// The HP cost of the test skill `e_bleed`.
const COST: i32 = 10;

/// Engine skills and the test skills: a jab that staggers (poise breaks), a heavy, a guard, a focus, a mend, an area
/// burst, a skill with an HP cost.
fn catalog() -> Catalog {
    let attack = |id: &str, blocks: i32, extra: Value| {
        let mut row = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": blocks, "power_mult": 1.0, "target_mode": "ENEMY"});
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
        attack("e_jab", 1, json!({"stagger": 30})),
        attack("e_heavy", 3, json!({"power_mult": 1.6})),
        attack("e_burst", 1, json!({"stagger": 10, "area": "PLUS"})),
        attack("e_bleed", 1, json!({"hp_cost": COST})),
        json!({"id": "e_guard", "name": "e_guard", "kind": "DEFEND", "level": 1, "blocks": 2, "mitigation": 0.5}),
        json!({"id": "e_focus", "name": "e_focus", "kind": "FOCUS", "level": 1, "blocks": 1, "boost": 0.1, "boost_rounds": 1}),
        json!({"id": "e_mend", "name": "e_mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY", "heal_base": 6}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 60, "speed": speed, "attack_range": 1, "skill_cap": 5, "poise": 40});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("e_hero", "party", 14), unit("e_ally", "party", 12), unit("e_foe", "enemy", 10), unit("e_foe_b", "enemy", 8)]
    }))
}

fn session(ruleset: Ruleset, seed: i64) -> Session {
    let row = json!({"id": "e_fight", "name": "Fight", "party": ["e_hero", "e_ally"], "enemies": ["e_foe", "e_foe_b"],
        "total_blocks": 6, "unit_ap": 10, "ap_regen": 4, "max_rounds": 12, "rng_seed": seed});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    Session::start(Arc::new(catalog()), def, &IndexMap::new())
}

fn timeline(rules: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap())
}

/// Plays `s` to its end through the auto driver (the host's `autoplay`).
fn play(s: &mut Session) {
    for _ in 0..40 {
        if s.combat_over {
            break;
        }
        if s.phase == Phase::Replay {
            if !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
        if !s.plan_confirmed {
            s.auto_draft("aggressive");
            assert!(s.confirm_plan(), "the auto plan confirms");
        }
        s.resolve_round();
    }
}

/// The tag of a log line (`[Attack] ...` is "Attack").
fn tag(line: &str) -> &str {
    line.strip_prefix('[').and_then(|l| l.split_once(']')).map_or("", |(t, _)| t)
}

/// What a line of each event kind says (the test reads the prose only to check the events against it).
fn describes(kind: &EventKind, line: &str, s: &Session) -> bool {
    match kind {
        EventKind::Hit { crit, zero } => {
            let area_zero = line.contains(" damage (") && line.contains(" for 0 damage");
            tag(line) == "Attack" && line.contains(" CRIT") == *crit && (*zero == (line.contains(" for 0 ") || area_zero))
        }
        EventKind::Miss => matches!(tag(line), "Attack Miss" | "Area Miss"),
        EventKind::Dodged => tag(line) == "Dodged",
        EventKind::Status { status, landed } if status.is_empty() => tag(line) == "Status" && !landed && !line.contains("(inflicts ") && !line.contains("(resisted "),
        EventKind::Status { status, landed } => tag(line) == "Status" && line.contains(&format!("({} {status})", if *landed { "inflicts" } else { "resisted" })),
        EventKind::LostBlock { status } => tag(line) == "Status" && line.contains(&format!(" is {status} and loses this block")),
        EventKind::Guard { broken } => tag(line) == "Defend" && line.contains("guard was broken") == *broken,
        EventKind::Focus => tag(line) == "Focus",
        EventKind::Support => tag(line) == "Support",
        EventKind::Move => matches!(tag(line), "Move" | "Move Cut Short" | "Withdrawal" | "Disengage" | "Dash" | "Chase"),
        EventKind::Break { unit } => tag(line) == "Break" && line.starts_with(&format!("[Break] {}", s.w.units[*unit].name)),
    }
}

/// Every event names a line it describes, at most one event a line, and every `[Attack]`, `[Break]`, `[Defend]`,
/// `[Focus]`, `[Support]` and move line has one.
fn check_events(s: &Session, counts: &mut IndexMap<&'static str, usize>) {
    let log = &s.resolver.log;
    let mut seen = std::collections::BTreeSet::new();
    for e in &s.resolver.events {
        let line = log.get(e.line).unwrap_or_else(|| panic!("event {:?} past the log ({} lines)", e, log.len()));
        assert!(describes(&e.kind, line, s), "{:?} does not describe {line:?}", e.kind);
        assert!(seen.insert(e.line), "two events on line {}: {line:?}", e.line);
        let key = match &e.kind {
            EventKind::Hit { crit: true, .. } => "crit",
            EventKind::Hit { zero: true, .. } => "zero",
            EventKind::Hit { .. } => "hit",
            EventKind::Miss => "miss",
            EventKind::Break { .. } => "break",
            EventKind::Guard { .. } => "guard",
            EventKind::Move => "move",
            _ => "other",
        };
        *counts.entry(key).or_insert(0) += 1;
    }
    for (i, line) in log.iter().enumerate() {
        if matches!(tag(line), "Attack" | "Break" | "Defend" | "Focus" | "Support" | "Move" | "Move Cut Short" | "Withdrawal" | "Disengage" | "Dash" | "Chase") {
            assert!(seen.contains(&i), "line {i} has no event: {line:?}");
        }
    }
    // `since` takes the events of the lines from an index on
    let half = log.len() / 2;
    assert_eq!(event::since(&s.resolver.events, half).count(), s.resolver.events.iter().filter(|e| e.line >= half).count());
}

#[test]
fn every_event_describes_its_log_line_and_every_reacting_line_has_one() {
    let mut counts = IndexMap::new();
    for seed in 1..=6 {
        for rules in [Ruleset::classic(), timeline(&[POISE])] {
            let mut s = session(rules, seed);
            play(&mut s);
            check_events(&s, &mut counts);
        }
    }
    assert!(counts.get("hit").copied().unwrap_or(0) > 0, "{counts:?}");
    assert!(counts.get("break").copied().unwrap_or(0) > 0, "the stagger-30 jab breaks a bar of 40: {counts:?}");
}

/// The events change nothing: the same fight's log is the same text whoever reads it, and replaying it gives the same
/// events.
#[test]
fn the_events_are_deterministic_and_leave_the_log_text_alone() {
    let mut a = session(timeline(&[POISE]), 3);
    let mut b = session(timeline(&[POISE]), 3);
    play(&mut a);
    play(&mut b);
    assert_eq!(a.resolver.log, b.resolver.log);
    assert_eq!(a.resolver.events, b.resolver.events);
    assert!(a.resolver.events.len() <= a.resolver.log.len());
}

/// A refusal's text is its fields' (`refusal::text`; the old strings: `command::hp_cost_problem`,
/// `meter::gate_problem`'s "<name> <have> < <min>"), and its JSON reads back.
#[test]
fn a_refusals_fields_give_its_text_and_read_back_from_json() {
    let hp = command::hp_cost_reason(9, COST).expect("at or below the cost");
    assert_eq!(hp, Reason::HpCost { hp: 9, cost: COST });
    assert_eq!(hp.to_string(), command::hp_cost_problem(9, COST));
    assert_eq!(command::hp_cost_reason(COST + 1, COST), None);
    assert_eq!(command::hp_cost_problem(COST + 1, COST), "");
    let gate = Reason::MeterGate { meter: "m".into(), name: "Gauge".into(), have: 30, min: 50 };
    assert_eq!(gate.to_string(), "Gauge 30 < 50");
    let both = vec![hp.clone(), gate.clone()];
    assert_eq!(refusal::text(&both), format!("{hp}{}{gate}", rip::REASONS_JOIN));
    assert_eq!(refusal::text(&[]), "");
    for r in [hp, gate, Reason::Text("Out of reach".into())] {
        assert_eq!(Reason::from_json(&r.to_json()), Some(r));
    }
    assert_eq!(Reason::from_json(&json!({"kind": "nope"})), None);
}

/// HOOKS1 HC1 through the planner: a step refused for its HP cost carries it in fields (`Plan::add_step_why`), and
/// `add_step`'s text is the same reason's.
#[test]
fn the_planner_refuses_an_hp_cost_in_fields() {
    let mut s = session(timeline(&[HP_COST]), 1);
    s.w.units[HERO].hp = COST;
    s.w.grid.positions[HERO] = Some(V2::new(4, 4));
    s.w.grid.positions[FOE] = Some(V2::new(5, 4));
    let bleed = s.w.book().find("e_bleed").expect("the test skill");
    let cost = command::hp_cost_for(&s.w, HERO, &s.w.cat.skills[bleed]);
    assert_eq!(cost, COST, "the flat HP cost");
    let mut plan = Plan::new();
    let why = plan.add_step_why(&s.w, HERO, PlanStep::engage(bleed, FOE), true);
    assert_eq!(why, vec![Reason::HpCost { hp: COST, cost }]);
    assert_eq!(plan.add_step(&s.w, HERO, PlanStep::engage(bleed, FOE), true), command::hp_cost_problem(COST, cost));
    assert!(plan.steps_of(HERO).is_empty(), "nothing added");
    s.w.units[HERO].hp = COST + 1;
    assert!(plan.add_step_why(&s.w, HERO, PlanStep::engage(bleed, FOE), true).is_empty(), "above the cost it is added");
}

/// A4.2-K1b: an area entry's marks take the Assault row when locked on a target, else the round's intent; a single-target
/// entry or one with no tiles has none.
#[test]
fn an_area_entry_is_marked_in_the_assault_row_when_locked_else_the_rounds() {
    let mut s = session(timeline(&[]), 1);
    s.w.grid.positions[HERO] = Some(V2::new(4, 4));
    s.w.grid.positions[FOE] = Some(V2::new(5, 4));
    let w = &s.w;
    let burst = w.book().find("e_burst").expect("the area skill");
    let jab = w.book().find("e_jab").expect("the jab");
    // a planned jab's entry, as the planner builds it, turned into the cases below
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(jab, FOE), true), "");
    let planned: PlanEntry = plan.entries_for(w, HERO).into_iter().find(|e| e.skill == jab).expect("the jab's entry");
    let mut e = PlanEntry { skill: burst, target: None, cells: Vec::new(), ..planned.clone() };
    assert_eq!(rip::area_mark(w, &e, RoundIntent::Field), None, "no tiles");
    e.cells = vec![V2::new(3, 3), V2::new(3, 4)];
    assert_eq!(rip::area_mark(w, &e, RoundIntent::Field), Some(RoundIntent::Field));
    e.target = Some(FOE);
    assert_eq!(rip::area_mark(w, &e, RoundIntent::Field), Some(RoundIntent::Assault), "locked on a target");
    let single = PlanEntry { cells: vec![V2::new(3, 3)], ..planned };
    assert_eq!(rip::area_mark(w, &single, RoundIntent::Field), None, "not an area skill");
    let marks = rip::area_marks(w, [&e, &single], RoundIntent::Field);
    assert_eq!(marks, vec![(e.cells.clone(), RoundIntent::Assault)]);
}
