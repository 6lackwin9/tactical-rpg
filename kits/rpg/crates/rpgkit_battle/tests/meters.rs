//! HOOKS1-M on a scratch catalog built here (no shipped data): unit meters (HOOKS1 9). Expectations come from the rules
//! and the rows below: a passive `meter` effect gives a meter that starts at the larger of its start and the holder's
//! floor (UM1); a skill's `meters` change it at the command's start, clamped to floor..max, a negative delta the value
//! does not cover is refused by the planners and fails unpaid when forced (UM2); decay rows run at round end
//! (`unless_gained`, `if_guarded`; UM3); a band adds its effects and holds its statuses only while the value is in it
//! (UM4); a rule fires on a crossing in its direction, a `once` rule once a battle, and a rule's own meter change
//! crosses no further rule (UM5); the row's skills are open to a holder without a slot while their cost is met and
//! FU4's auto order lists them after the deck's actives (UM6); the trigger effect `meter` and condition `meter` (8.2).
//! In a session the meters reach the snapshots, the log and the metrics (UM7). With the rule off nothing changes.

use rpgkit_battle::command::Command;
use rpgkit_battle::meter;
use rpgkit_battle::metrics::BattleMetrics;
use rpgkit_battle::plan::{Plan, PlanStep, Step};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Roster, Session};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::{Catalog, PassiveDef, UnitDef};
use rpgkit_rules::ruleset::{Ruleset, TIMELINE, TRIGGERS, UNIT_METERS};
use rpgkit_rules::{status, Overrides};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
/// The test rows' numbers (the expectations derive from them).
const PHIAL_MAX: i32 = 3;
const STRAIN_START: i32 = 50;
const STRAIN_GAIN: i32 = 20;
const DECAY: i32 = 10;
const HOT_FROM: i32 = 2;
const HOT_EMPOWER: i32 = 10;
/// Above the band's empower, so a test can tell which of the two is held.
const GRANT_EMPOWER: i32 = 15;
const HOT_CRIT: f64 = 5.0;
const HOT_RESIST: i32 = 50;
const TIDE_START: i32 = 2;
const TIDE_STEP: i32 = 2;
const CRISIS_START: i32 = 8;
const CRISIS_SET: i32 = 7;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;

fn focus(id: &str, name: &str, meters: Value) -> Value {
    json!({"id": id, "name": name, "kind": "FOCUS", "level": 1, "blocks": 1, "ap_cost": 1, "boost": 0.1, "boost_rounds": 1, "meters": meters})
}

/// Engine skills, a jab, a mend, and the meter skills: Coat (the phial's meter skill, learned only: a unit owns it
/// only as a meter skill), Inoculate (+20 strain), Heat / Vent (fury +2 / -1), Rise / Fall (tide and surge +2 / -2),
/// Spike (crisis +2). Meters: phial 0-3; strain 0-100 from 50 with two decay rows; fury 0-5 with a hot band from 2;
/// tide and surge 0-10 from 2 with a rule at 3 (tide's `once`); crisis 0-10 from 8 whose rule at 10 sets it to 7
/// (crossing a down rule at 9 that must not fire).
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
        json!({"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "heal_base": 5, "target_mode": "ALLY"}),
        json!({"id": "t_coat", "name": "Coat", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "heal_base": 5, "target_mode": "ALLY", "learned_only": true,
            "meters": [{"meter": "phial", "delta": -1}]}),
        json!({"id": "t_empower", "name": "Empower", "kind": "SUPPORT", "level": 1, "blocks": 1, "ap_cost": 0, "reach_min": 1, "reach_max": 2, "target_mode": "ALLY", "learned_only": true,
            "grant_status": "empower", "status_magnitude": GRANT_EMPOWER, "status_rounds": 1, "meters": [{"meter": "salve", "delta": -1}]}),
        json!({"id": "t_inoc", "name": "Inoculate", "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5, "target_mode": "ENEMY",
            "meters": [{"meter": "strain", "delta": STRAIN_GAIN}]}),
        focus("t_heat", "Heat", json!([{"meter": "fury", "delta": 2}])),
        focus("t_vent", "Vent", json!([{"meter": "fury", "delta": -1}])),
        focus("t_rise", "Rise", json!([{"meter": "tide", "delta": TIDE_STEP}, {"meter": "surge", "delta": TIDE_STEP}])),
        focus("t_fall", "Fall", json!([{"meter": "tide", "delta": -TIDE_STEP}, {"meter": "surge", "delta": -TIDE_STEP}])),
        focus("t_spike", "Spike", json!([{"meter": "crisis", "delta": 2}])),
    ];
    let peak = |once: bool| json!([{"id": "peak", "at": 3, "dir": "up", "once": once, "effects": [{"type": "crit", "value": 1}]}]);
    let meters = vec![
        json!({"id": "phial", "name": "Phial", "short": "Ph", "min": 0, "max": PHIAL_MAX, "start": 0, "show": "pips", "skills": ["t_coat"]}),
        json!({"id": "salve", "name": "Salve", "short": "Sa", "min": 0, "max": PHIAL_MAX, "start": 0, "show": "pips", "skills": ["t_empower"]}),
        json!({"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "start": STRAIN_START, "show": "bar",
            "decay": [{"delta": -DECAY, "unless_gained": true}, {"delta": -DECAY, "unless_gained": true, "if_guarded": true}]}),
        json!({"id": "fury", "name": "Fury", "short": "Fu", "min": 0, "max": 5, "start": 0, "show": "pips", "bands": [
            {"id": "calm", "name": "Calm", "from": 0},
            {"id": "hot", "name": "Hot", "from": HOT_FROM, "effects": [{"type": "crit", "value": HOT_CRIT}, {"type": "resist", "status": "stun", "value": HOT_RESIST}],
                "statuses": [{"status": "empower", "magnitude": HOT_EMPOWER}]}]}),
        json!({"id": "tide", "name": "Tide", "short": "Ti", "min": 0, "max": 10, "start": TIDE_START, "show": "pips", "rules": peak(true)}),
        json!({"id": "surge", "name": "Surge", "short": "Su", "min": 0, "max": 10, "start": TIDE_START, "show": "pips", "rules": peak(false)}),
        json!({"id": "crisis", "name": "Crisis", "short": "Cr", "min": 0, "max": 10, "start": CRISIS_START, "show": "hidden", "rules": [
            {"id": "burst", "at": 10, "effects": [{"type": "meter_set", "meter": "crisis", "value": CRISIS_SET}, {"type": "crit", "value": 1}]},
            {"id": "ebb", "at": 9, "dir": "down", "effects": [{"type": "crit", "value": 100}]}]}),
    ];
    let unit = |id: &str, side: &str, speed: i32| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 5000, "power": 100, "armor": 0, "speed": speed, "attack_range": 1, "skill_cap": 5,
            "deck": {"active": ["t_mend"]}})
    };
    Catalog::from_value(&json!({
        "skills": skills,
        "unit_meters": meters,
        "units": [unit("p_hero", "party", 14), unit("p_ally", "party", 12), unit("p_foe", "enemy", 10)],
    }))
}

/// Catalog units whose passives give the listed `meter` effects (ref -> effects), as a world character's would: passive
/// `trait<i>` per entry, added to the catalog (`with_traits`) and named in the ref's overrides (the one builder).
struct Holders(Vec<(&'static str, Value)>);

impl Holders {
    fn with_traits(&self, mut cat: Catalog) -> Catalog {
        for (i, (_, e)) in self.0.iter().enumerate() {
            let id = format!("trait{i}");
            cat.passives.insert(id.clone(), PassiveDef { id, name: format!("Trait {i}"), effects: vec![e.clone()], slot: String::new() });
        }
        cat
    }
}

impl Roster for Holders {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        cat.units.get(r).cloned()
    }

    fn overrides(&self, _cat: &Catalog, r: &str) -> Overrides {
        let passives = self.0.iter().enumerate().filter(|(_, (who, _))| *who == r).map(|(i, _)| format!("trait{i}")).collect();
        Overrides { passives, ..Overrides::default() }
    }
}

fn rules(list: &[&str]) -> Ruleset {
    Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": list}).as_object().unwrap())
}

fn holds(meter: &str) -> Value {
    json!({"type": "meter", "meter": meter})
}

/// Hero (4,4) faces the foe (5,4); the ally beside the hero at (4,5).
fn session(ruleset: Ruleset, held: Vec<(&'static str, Value)>) -> Session {
    let row = json!({"id": "t_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"],
        "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let holders = Holders(held);
    let mut s = Session::start(Arc::new(holders.with_traits(catalog())), def, &holders);
    for (c, cell) in [(HERO, rpgkit_battle::grid::V2::new(4, 4)), (ALLY, rpgkit_battle::grid::V2::new(4, 5)), (FOE, rpgkit_battle::grid::V2::new(5, 4))] {
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

fn own(w: &World, c: UnitId, skill: &str) -> Command {
    command(w, c, PlanStep::self_skill(sid(w, skill)), skill)
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

fn set(w: &mut World, c: UnitId, m: &str, v: i32) {
    w.meters.gauges.iter_mut().find(|g| g.unit == c && g.meter == m).expect("a gauge").value = v;
}

/// The lines of the meter `m`'s rules for `c` the resolver has not handed to a session yet.
fn rule_lines(res: &Resolver, c: &str, m: &str) -> usize {
    res.triggers.fired.iter().flat_map(|f| f.lines.iter()).filter(|l| l.starts_with(&format!("[Meter] {c}'s {m}: "))).count()
}

/// UM1, UM2: `start` 1 and `floor` 2 start at 2 (the larger); a holder without either starts at the row's 0. A cost of
/// 1 at 0 is refused (`Phial 0 < 1`); forced, the command fails unpaid (cancelled before its AP, nothing paid, logged);
/// at 1 it is paid at the start (0). A cost at the floor clamps there.
#[test]
fn a_meter_starts_at_its_floor_and_a_cost_needs_its_value() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", json!({"type": "meter", "meter": "phial", "start": 1, "floor": 2})), ("p_ally", holds("phial"))]);
    assert_eq!(meter::value(&s.w, HERO, "phial"), Some(2), "max(start 1, floor 2)");
    assert_eq!(meter::value(&s.w, ALLY, "phial"), Some(0), "the row's start");
    let coat = sid(&s.w, "t_coat");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&s.w, ALLY, PlanStep::ally(coat, HERO), true), "Phial 0 < 1");

    let mut w = s.w.clone();
    set(&mut w, ALLY, "phial", 1);
    let cmd = command(&w, ALLY, PlanStep::ally(coat, HERO), "t_coat");
    assert_eq!(cmd.meters.len(), 1);
    set(&mut w, ALLY, "phial", 0);
    w.units[ALLY].ap = 5;
    let mut res = Resolver::new(BLOCKS, 1);
    let a = book(&mut res, ALLY, cmd.clone(), 1);
    let order = res.prepare_block(&mut w, 1);
    assert!(res.scheduled[a].cancelled && !order.contains(&a), "failed before it ran");
    assert_eq!((w.units[ALLY].ap, meter::value(&w, ALLY, "phial")), (5, Some(0)), "nothing paid");
    assert!(res.log.iter().any(|l| l == "[Meter] p_ally's Coat: fails unpaid (Phial 0 < 1)"), "{:?}", res.log);

    let mut w = s.w.clone();
    set(&mut w, ALLY, "phial", 1);
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, ALLY, cmd, 1);
    res.prepare_block(&mut w, 1);
    assert_eq!(meter::value(&w, ALLY, "phial"), Some(0));
    assert!(res.log.iter().any(|l| l == "[Meter] p_ally's Coat: Phial -1 (1 -> 0)"), "{:?}", res.log);

    // the hero's floor is 2: paying 1 at 2 leaves 2
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, command(&w, HERO, PlanStep::ally(coat, ALLY), "t_coat"), 1);
    res.prepare_block(&mut w, 1);
    assert_eq!(meter::value(&w, HERO, "phial"), Some(2), "clamped to the floor");
}

/// UM2 in the planners: a second Coat planned after the first sees the value the first leaves; the round-intent fill
/// gives the cost as its cause.
#[test]
fn the_planner_counts_the_costs_of_earlier_steps() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_ally", holds("phial"))]);
    let mut w = s.w.clone();
    set(&mut w, ALLY, "phial", 1);
    let coat = sid(&w, "t_coat");
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, ALLY, PlanStep::ally(coat, HERO), true), "");
    assert_eq!(plan.compile(&w, ALLY).meter("phial"), Some(0));
    assert_eq!(plan.add_step(&w, ALLY, PlanStep::ally(coat, HERO), true), "Phial 0 < 1");
}

/// UM3: decay -10 `unless_gained` (twice; the second `if_guarded`): after a round a skill raised the meter, no decay;
/// after a quiet round -10; after a quiet round with a Guard -20.
#[test]
fn decay_skips_a_gaining_round_and_doubles_after_a_guard() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("strain"))]);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, command(&w, HERO, PlanStep::engage(sid(&w, "t_inoc"), FOE), "t_inoc"), 1);
    run(&mut w, &mut res, 1, BLOCKS);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(STRAIN_START + STRAIN_GAIN));
    res.apply_round_end_effects(&mut w);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(STRAIN_START + STRAIN_GAIN), "a gaining round");

    let mut res = Resolver::new(BLOCKS, 1);
    run(&mut w, &mut res, 1, BLOCKS);
    res.apply_round_end_effects(&mut w);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(STRAIN_START + STRAIN_GAIN - DECAY), "a quiet round");

    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, own(&w, HERO, "guard"), 1);
    run(&mut w, &mut res, 1, BLOCKS);
    res.apply_round_end_effects(&mut w);
    assert_eq!(meter::value(&w, HERO, "strain"), Some(STRAIN_START + STRAIN_GAIN - 3 * DECAY), "a quiet guarded round");
    assert!(res.log.iter().any(|l| l == &format!("[Meter] p_hero's Strain: decays -{DECAY} ({} -> {})", STRAIN_START + STRAIN_GAIN - DECAY, STRAIN_START + STRAIN_GAIN - 2 * DECAY)), "{:?}", res.log);
}

/// UM4: entering the hot band adds its crit and resist and grants its empower; the round start grants the empower
/// again after the roll-over drops it; leaving the band takes all of it off.
#[test]
fn a_band_holds_its_effects_and_statuses_only_while_in_it() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("fury"))]);
    let mut w = s.w.clone();
    let (crit, resist) = (w.units[HERO].trigger_crit, w.units[HERO].status_resist.get("stun").copied().unwrap_or(0));
    assert!(!w.units[HERO].has_status("empower"));
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, own(&w, HERO, "t_heat"), 1);
    run(&mut w, &mut res, 1, 1);
    let u = &w.units[HERO];
    assert_eq!(meter::value(&w, HERO, "fury"), Some(HOT_FROM));
    assert_eq!((u.active_magnitude("empower"), u.trigger_crit, u.status_resist.get("stun").copied().unwrap_or(0)), (HOT_EMPOWER, crit + HOT_CRIT, resist + HOT_RESIST));
    assert!(res.log.iter().any(|l| l == "[Meter] p_hero's Heat: p_hero's Fury enters Hot"), "{:?}", res.log);
    status::roll_over(&mut w.units[HERO], 0);
    assert!(!w.units[HERO].has_status("empower"), "a 1-round grant");
    meter::round_start(&mut w);
    assert_eq!(w.units[HERO].active_magnitude("empower"), HOT_EMPOWER, "held again at the round start");
    // planned once the heat has paid (a cost of 1 at 0 is refused)
    book(&mut res, HERO, own(&w, HERO, "t_vent"), 2);
    run(&mut w, &mut res, 2, 2);
    let u = &w.units[HERO];
    assert_eq!(meter::value(&w, HERO, "fury"), Some(HOT_FROM - 1));
    assert_eq!((u.has_status("empower"), u.trigger_crit, u.status_resist.get("stun").copied().unwrap_or(0)), (false, crit, resist), "left");
    // HOOKS1-UI (the HOOKS1-M review): a resist key the band created goes with it; one the unit had before stays
    assert!(!s.w.units[HERO].status_resist.contains_key("stun"), "the fixture hero has no stun resist of its own");
    assert!(!u.status_resist.contains_key("stun"), "the band's stun resist key went with it: {:?}", u.status_resist);
}

/// HOOKS1-UI slice 1 review: a resist key the unit held before the band (even at 0) stays when the band is left, at its
/// own value.
#[test]
fn a_band_on_a_resist_key_the_unit_had_keeps_the_key() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("fury"))]);
    for own_value in [0, 7] {
        let mut w = s.w.clone();
        w.units[HERO].status_resist.insert("stun".into(), own_value);
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, HERO, own(&w, HERO, "t_heat"), 1);
        run(&mut w, &mut res, 1, 1);
        assert_eq!(w.units[HERO].status_resist.get("stun").copied(), Some(own_value + HOT_RESIST), "in the band");
        meter::round_start(&mut w);
        book(&mut res, HERO, own(&w, HERO, "t_vent"), 2);
        run(&mut w, &mut res, 2, 2);
        assert_eq!(meter::value(&w, HERO, "fury"), Some(HOT_FROM - 1));
        assert_eq!(w.units[HERO].status_resist.get("stun").copied(), Some(own_value), "left: the unit's own key and value");
    }
}

/// UM5: a rule at 3 `up` over 2, 4, 2, 4 fires on each rise across it; a `once` rule only the first time.
#[test]
fn a_rule_fires_on_each_crossing_and_once_rules_once() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("tide")), ("p_hero", holds("surge"))]);
    let mut w = s.w.clone();
    let crit = w.units[HERO].trigger_crit;
    let mut res = Resolver::new(BLOCKS, 1);
    for (b, skill) in [(1, "t_rise"), (2, "t_fall"), (3, "t_rise"), (4, "t_fall")] {
        book(&mut res, HERO, own(&w, HERO, skill), b);
    }
    run(&mut w, &mut res, 1, 4);
    assert_eq!((meter::value(&w, HERO, "tide"), meter::value(&w, HERO, "surge")), (Some(TIDE_START), Some(TIDE_START)));
    assert_eq!((rule_lines(&res, "p_hero", "Tide"), rule_lines(&res, "p_hero", "Surge")), (1, 2));
    assert_eq!(w.units[HERO].trigger_crit, crit + 3.0, "once + twice");
}

/// UM5 and the depth-1 rule: a rise across 10 sets the meter to 7 (`meter_set`) and its crit runs; the set falls across
/// the down rule at 9, which does not fire.
#[test]
fn a_rules_own_meter_change_crosses_no_further_rule() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("crisis"))]);
    let mut w = s.w.clone();
    let crit = w.units[HERO].trigger_crit;
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, HERO, own(&w, HERO, "t_spike"), 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(meter::value(&w, HERO, "crisis"), Some(CRISIS_SET));
    assert_eq!(w.units[HERO].trigger_crit, crit + 1.0, "burst only, not ebb");
    assert!(meter::shown(&w, HERO).is_empty(), "a hidden meter is not shown");
}

/// UM6: the phial's Coat (learned only) is open to its holder without a slot while the phial covers it: offered at 1,
/// not at 0; FU4's auto order lists it after the deck's active (Mend). Without the rule nobody owns it.
#[test]
fn a_meter_skill_is_offered_slotless_while_its_cost_is_met() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_ally", holds("phial"))]);
    let mut w = s.w.clone();
    let (coat, mend) = (sid(&w, "t_coat"), sid(&w, "t_mend"));
    assert!(w.units[ALLY].knows_skill(&w.cat, "t_coat") && !w.units[HERO].knows_skill(&w.cat, "t_coat"));
    assert!(meter::offered(&w, ALLY).is_empty(), "the phial is empty");
    set(&mut w, ALLY, "phial", 1);
    assert_eq!(meter::offered(&w, ALLY), vec!["t_coat".to_string()]);
    assert_eq!(rip::order(&w, ALLY, RoundIntent::Support), vec![mend, coat], "after the deck's active");
    assert_eq!(rip::order(&w, HERO, RoundIntent::Support), vec![mend]);

    let off = session(rules(&[]), vec![("p_ally", holds("phial"))]);
    assert!(off.w.meters.gauges.is_empty() && off.w.units[ALLY].meter_skills.is_empty());
    assert!(!off.w.units[ALLY].knows_skill(&off.w.cat, "t_coat"));
}

/// 8.2: a `hit_dealt` trigger's `meter` +1 raises the holder's phial on each hit; a second trigger with `when.meter`
/// `{phial, at_least 2}` fires only once the phial reaches 2. Without the rule unit_meters neither the effect nor the
/// condition does anything.
#[test]
fn the_trigger_meter_effect_and_condition() {
    for on in [true, false] {
        let list: &[&str] = if on { &[UNIT_METERS, TRIGGERS] } else { &[TRIGGERS] };
        let s = session(rules(list), vec![("p_hero", holds("phial"))]);
        let mut w = s.w.clone();
        let cat = w.cat.clone();
        let fill = json!({"type": "trigger", "on": "hit_dealt", "when": {}, "effects": [{"type": "meter", "meter": "phial", "delta": 1}], "limit": "none"});
        let brim = json!({"type": "trigger", "on": "hit_dealt", "when": {"meter": {"id": "phial", "at_least": 2}}, "effects": [{"type": "crit", "value": 1}], "limit": "none"});
        w.units[HERO].apply_effects(&cat, &[fill, brim], "passive:brew", false);
        let crit = w.units[HERO].trigger_crit;
        let mut res = Resolver::new(BLOCKS, 1);
        let jab = sid(&w, "t_jab");
        book(&mut res, HERO, command(&w, HERO, PlanStep::engage(jab, FOE), "t_jab"), 1);
        book(&mut res, HERO, command(&w, HERO, PlanStep::engage(jab, FOE), "t_jab"), 2);
        run(&mut w, &mut res, 1, 1);
        if on {
            assert_eq!((meter::value(&w, HERO, "phial"), w.units[HERO].trigger_crit), (Some(1), crit));
            assert!(res.log.iter().any(|l| l == "[Trigger] p_hero's brew: p_hero's Phial +1 (0 -> 1)"), "{:?}", res.log);
        }
        run(&mut w, &mut res, 2, 2);
        if on {
            assert_eq!((meter::value(&w, HERO, "phial"), w.units[HERO].trigger_crit), (Some(2), crit + 1.0));
        } else {
            assert_eq!((meter::value(&w, HERO, "phial"), w.units[HERO].trigger_crit), (None, crit));
        }
    }
}

/// Rule off (and classic): no gauge, no meter cost on a command, no refusal, no `[Meter]` line.
#[test]
fn without_the_rule_nothing_changes() {
    for ruleset in [Ruleset::classic(), rules(&[TRIGGERS])] {
        let s = session(ruleset, vec![("p_hero", holds("fury"))]);
        let mut w = s.w.clone();
        assert!(w.meters.gauges.is_empty());
        let vent = own(&w, HERO, "t_vent");
        assert!(vent.meters.is_empty());
        let mut res = Resolver::new(BLOCKS, 1);
        book(&mut res, HERO, vent, 1);
        run(&mut w, &mut res, 1, BLOCKS);
        res.apply_round_end_effects(&mut w);
        assert!(res.log.iter().all(|l| !l.starts_with("[Meter]")), "{:?}", res.log);
        assert!(s.snapshot().iter().all(|u| u.meters.is_empty()));
    }
}

/// UM7 and 4.2 in a session: the snapshot and `shown` carry the meter (pips, its band); a planned Heat logs its meter
/// cells (category EVENT, subtype Trigger, the skill's name) and the metrics count them under `triggers.meters`.
#[test]
fn a_session_shows_logs_and_counts_the_meters() {
    let mut s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("fury"))]);
    assert_eq!(s.snapshot()[HERO].meters, vec![("fury".to_string(), 0)]);
    let shown = meter::shown(&s.w, HERO);
    assert_eq!((shown.len(), shown[0].show.as_str(), shown[0].band.as_str(), shown[0].short.as_str()), (1, "pips", "Calm", "Fu"));
    s.attach_metrics(BattleMetrics::new(), true);
    assert_eq!(s.plan.add_step(&s.w, HERO, PlanStep::self_skill(sid(&s.w, "t_heat")), true), "");
    while s.current_round == 1 && !s.combat_over && s.phase != Phase::Replay {
        s.step_next();
    }
    assert_eq!(meter::value(&s.w, HERO, "fury"), Some(HOT_FROM));
    let cells: Vec<_> = s.log.iter().filter(|e| e.skill == "Heat" && e.subtype == "Trigger").collect();
    assert_eq!(cells.len(), 2, "the change and the band");
    assert_eq!((cells[0].category, cells[0].actor.as_str()), ("EVENT", "p_hero"));
    assert_eq!(s.snapshot()[HERO].meters, vec![("fury".to_string(), HOT_FROM)]);
    let mut m = s.metrics.take().unwrap();
    m.finish(&s);
    assert!(m.fail_lines.is_empty(), "{:?}", m.fail_lines);
    assert_eq!(m.triggers["p_hero"].meters, 1);
}

/// EMPOWER1: a meter skill grants empower to an ally like any support buff: queued, active after the roll-over at the
/// skill's magnitude, raising outgoing power by it, gone when its 1 round ends; the cost is paid.
#[test]
fn a_skill_grants_empower_for_its_rounds() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_ally", holds("salve"))]);
    let mut w = s.w.clone();
    set(&mut w, ALLY, "salve", 1);
    let id = sid(&w, "t_empower");
    let cmd = command(&w, ALLY, PlanStep::ally(id, HERO), "t_empower");
    let mut res = Resolver::new(BLOCKS, 1);
    book(&mut res, ALLY, cmd, 1);
    run(&mut w, &mut res, 1, 1);
    assert_eq!(meter::value(&w, ALLY, "salve"), Some(0));
    assert!(w.units[HERO].st.has_any("empower") && !w.units[HERO].has_status("empower"), "queued for next round");
    status::roll_over(&mut w.units[HERO], 0);
    assert_eq!(w.units[HERO].active_magnitude("empower"), GRANT_EMPOWER);
    assert!((w.units[HERO].empower_factor() - (1.0 + GRANT_EMPOWER as f64 / 100.0)).abs() < 1e-9);
    status::roll_over(&mut w.units[HERO], 0);
    assert!(!w.units[HERO].st.has_any("empower"), "a 1-round grant");
}

/// Puts the hero in the hot band.
fn heat(w: &mut World, res: &mut Resolver, block: i32) {
    book(res, HERO, own(w, HERO, "t_heat"), block);
    run(w, res, block, block);
    assert_eq!(meter::value(w, HERO, "fury"), Some(HOT_FROM));
}

/// Takes the hero out of the hot band.
fn vent(w: &mut World, res: &mut Resolver, block: i32) {
    book(res, HERO, own(w, HERO, "t_vent"), block);
    run(w, res, block, block);
    assert_eq!(meter::value(w, HERO, "fury"), Some(HOT_FROM - 1));
}

/// EMPOWER1, coat then band: the higher applies while both hold; leaving the band gives back the grant's own numbers.
#[test]
fn a_grant_outlives_a_band_entered_after_it() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("fury"))]);
    for (grant, merged) in [(GRANT_EMPOWER, GRANT_EMPOWER), (HOT_EMPOWER - 1, HOT_EMPOWER)] {
        let mut w = s.w.clone();
        let mut res = Resolver::new(BLOCKS, 1);
        status::grant_now(&mut w.units[HERO], "empower", 3, grant, "coat");
        heat(&mut w, &mut res, 1);
        assert_eq!(w.units[HERO].active_magnitude("empower"), merged);
        vent(&mut w, &mut res, 2);
        assert_eq!(w.units[HERO].active_magnitude("empower"), grant, "the grant alone");
        assert_eq!(w.units[HERO].st.rounds_left.get("empower"), Some(&3));
        assert_eq!(w.units[HERO].st.sources.get("empower").map(String::as_str), Some("coat"));
    }
}

/// EMPOWER1, band then coat, active: the grant stays at its own magnitude after the band goes, and counts down.
#[test]
fn a_grant_outlives_a_band_it_landed_on() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("fury"))]);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    heat(&mut w, &mut res, 1);
    status::grant_now(&mut w.units[HERO], "empower", 2, HOT_EMPOWER - 1, "coat");
    assert_eq!(w.units[HERO].active_magnitude("empower"), HOT_EMPOWER);
    status::roll_over(&mut w.units[HERO], 0);
    meter::round_start(&mut w);
    assert_eq!(w.units[HERO].active_magnitude("empower"), HOT_EMPOWER);
    vent(&mut w, &mut res, 2);
    assert_eq!(w.units[HERO].active_magnitude("empower"), HOT_EMPOWER - 1);
    assert_eq!(w.units[HERO].st.rounds_left.get("empower"), Some(&1), "one round already spent");
}

/// EMPOWER1, queued: a band change leaves a queued grant queued; queued while a band holds, it arrives merged and
/// survives the band.
#[test]
fn a_queued_grant_outlives_a_band() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("fury"))]);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    heat(&mut w, &mut res, 1);
    status::queue(&mut w.units[HERO], "empower", 1, GRANT_EMPOWER, "coat");
    vent(&mut w, &mut res, 2);
    assert!(!w.units[HERO].has_status("empower") && w.units[HERO].st.pending.contains("empower"), "the band's part went, the queue stayed");
    status::roll_over(&mut w.units[HERO], 0);
    assert_eq!(w.units[HERO].active_magnitude("empower"), GRANT_EMPOWER);

    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    heat(&mut w, &mut res, 1);
    status::queue(&mut w.units[HERO], "empower", 2, HOT_EMPOWER - 1, "coat");
    status::roll_over(&mut w.units[HERO], 0);
    meter::round_start(&mut w);
    assert_eq!(w.units[HERO].active_magnitude("empower"), HOT_EMPOWER);
    vent(&mut w, &mut res, 2);
    assert_eq!(w.units[HERO].active_magnitude("empower"), HOT_EMPOWER - 1);
}

/// EMPOWER1: a grant ending never removes the band's part: after it runs out the band holds again at its own number.
#[test]
fn a_grant_ending_leaves_the_band() {
    let s = session(rules(&[UNIT_METERS]), vec![("p_hero", holds("fury"))]);
    let mut w = s.w.clone();
    let mut res = Resolver::new(BLOCKS, 1);
    heat(&mut w, &mut res, 1);
    status::grant_now(&mut w.units[HERO], "empower", 1, GRANT_EMPOWER, "coat");
    assert_eq!(w.units[HERO].active_magnitude("empower"), GRANT_EMPOWER);
    status::roll_over(&mut w.units[HERO], 0);
    meter::round_start(&mut w);
    assert_eq!(w.units[HERO].active_magnitude("empower"), HOT_EMPOWER);
    vent(&mut w, &mut res, 2);
    assert!(!w.units[HERO].st.has_any("empower"), "nothing left of either");
}
