//! A4.2-K1b (W2-4 slice 6a; INTENT2 IR2-IR7, IR10, IR15, IR17, IR19, IR21; `docs/sessions/A4_EFFECTS.md` section 2,
//! `A3_SHAPES.md` sections 2-3) on a scratch catalog built here (no shipped data): the Field round intent and the
//! row-aware planner under the rule `field_intent`. A dual skill (IR21 `Field, Assault`) is offered in both rows: the
//! Field row aims at a source tile fixed at planning (one source a round, each family once), with the derived Field
//! bonus (`area`: ARC 5 across, LINE 4 long; `reach`: +1; `lingering`: +1 hazard round); the Assault row is
//! target-locked (its shape re-read on the target's tile at the impact), takes `splash_mult` off its target, lands its
//! status on the target only, leaves no hazard and winds up `assault_shape_blocks` longer. Without the rule the legacy
//! single intent holds and nothing changes.

use indexmap::IndexMap;
use rpgkit_battle::ai;
use rpgkit_battle::command::{CmdKind, Command};
use rpgkit_battle::grid::V2;
use rpgkit_battle::plan::{Plan, PlanEntry, PlanStep};
use rpgkit_battle::resolver::Resolver;
use rpgkit_battle::round_intent::{self, Pick, RoundIntent, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::skill;
use rpgkit_battle::world::{Tactic, UnitId, UnitTactic, World};
use rpgkit_data::Catalog;
use rpgkit_rules::intent::FieldBonus;
use rpgkit_rules::ruleset::{self, Ruleset, AI_TACTICS, FIELD_INTENT, GROUND_HAZARDS, ROUND_INTENT, TIMELINE};
use serde_json::{json, Value};
use std::sync::Arc;

const BLOCKS: i32 = 6;
const HP: i32 = 5000;
const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;
const FOE2: UnitId = 3;
const FOE3: UnitId = 4;
const HERO_AT: V2 = V2::new(2, 4);
/// The foe 3 tiles from the hero (in a BURST's band 2-3), the second foe beside it, the third and the ally away.
const FOE_AT: V2 = V2::new(5, 4);
const FOE2_AT: V2 = V2::new(5, 5);
const FOE3_AT: V2 = V2::new(9, 8);
const ALLY_AT: V2 = V2::new(0, 0);

fn catalog() -> Catalog {
    let row = |id: &str, extra: Value| {
        let mut r = json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "power_mult": 0.5});
        r.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        r
    };
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        // a thrown BURST (IR21 Field, Assault; Field bonus `reach`), with a status
        row("f_burst", json!({"blocks": 2, "reach_min": 2, "reach_max": 3, "area": "BURST", "status_effect": "weaken", "status_potency": 100, "status_rounds": 1})),
        // a drawn ARC and LINE (Field, Assault; Field bonus `area`)
        row("f_arc", json!({"blocks": 2, "area": "ARC"})),
        row("f_line", json!({"blocks": 2, "area": "LINE"})),
        // a thrown BURST with a hazard (Field only; Field bonus `lingering`)
        row("f_pot", json!({"blocks": 2, "reach_min": 2, "reach_max": 3, "area": "BURST", "hazard": {"id": "f_fire", "shape": "SKILL"}})),
        // a whole-round BURST: the Assault row's extra block cannot fit past the round
        row("f_long", json!({"blocks": 6, "reach_min": 2, "reach_max": 3, "area": "BURST"})),
        // a reach-1 BURST with no hazard (IR21: Field only; Field bonus `power`)
        row("f_near", json!({"blocks": 2, "reach_min": 1, "reach_max": 1, "area": "BURST"})),
        // a RING (IR21: Stand only), drawn around the user
        row("f_ring", json!({"blocks": 2, "area": "RING"})),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": HP, "power": 20, "armor": 0, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({
        "skills": skills,
        "units": [unit("p_hero", "party"), unit("p_ally", "party"), unit("p_foe", "enemy"), unit("p_foe2", "enemy"), unit("p_foe3", "enemy")],
        "hazards": [{"id": "f_fire", "name": "Fire", "on": "round_end", "damage": 1, "duration": 2}]
    }))
}

fn rules(list: &[&str], extra: Value) -> Ruleset {
    let mut row = json!({"id": TIMELINE, "name": "Timeline", "rules": list});
    row.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    Ruleset::from_row(row.as_object().unwrap())
}

fn field_on() -> Ruleset {
    rules(&[ROUND_INTENT, FIELD_INTENT], json!({}))
}

fn session(ruleset: Ruleset) -> World {
    let row = json!({"id": "f_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe", "p_foe2", "p_foe3"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = ruleset;
    let mut s = Session::start(Arc::new(catalog()), def, &IndexMap::new());
    for (c, cell) in [(HERO, HERO_AT), (ALLY, ALLY_AT), (FOE, FOE_AT), (FOE2, FOE2_AT), (FOE3, FOE3_AT)] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.w.clone()
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// `c`'s round planned by the auto planner under `intent` on `target` (opener `opener` when given).
fn planned(w: &mut World, intent: RoundIntent, target: RoundTarget, opener: Option<&str>) -> (Plan, Result<(), String>) {
    let mut plan = Plan::new();
    round_intent::set_intent(w, HERO, intent).expect("intent");
    round_intent::set_target(w, HERO, target).expect("target");
    if let Some(o) = opener {
        let id = sid(w, o);
        rip::set_opener(w, HERO, Some(round_intent::Opener::Skill(id))).unwrap();
    }
    let r = rip::plan_unit(w, &mut plan, HERO);
    (plan, r)
}

fn entries_of(w: &World, plan: &Plan, id: &str) -> Vec<PlanEntry> {
    let s = sid(w, id);
    plan.entries_for(w, HERO).into_iter().filter(|e| e.skill == s).collect()
}

/// Resolves `cmd` alone from block 1; `between` runs after the start block (a foe may step before the impact).
fn resolve(w: &mut World, cmd: Command, between: impl Fn(&mut World)) -> Command {
    let mut res = Resolver::new(BLOCKS, 1);
    let impact = cmd.block_cost;
    assert!(res.schedule(HERO, cmd, 1), "booked");
    for b in 1..=impact {
        let due = res.prepare_block(w, b);
        if b == 1 {
            between(w);
        }
        for i in due {
            res.execute_single(w, i);
        }
    }
    res.scheduled[0].command.clone()
}

// ---- IR2, IR5: Field is a tile intent, offered by the rule ---------------------------------------------------------

#[test]
fn field_is_offered_only_under_the_rule_and_aims_at_a_tile() {
    for off in [Ruleset::classic(), rules(&[ROUND_INTENT], json!({}))] {
        let mut w = session(off.clone());
        assert!(!round_intent::listed(&w, RoundIntent::Field), "{:?}", off.rules);
        assert!(round_intent::set_intent(&mut w, HERO, RoundIntent::Field).is_err(), "no Field without field_intent");
    }
    let mut w = session(field_on());
    assert!(round_intent::listed(&w, RoundIntent::Field));
    assert_eq!(RoundIntent::parse("field"), Some(RoundIntent::Field));
    assert_eq!(RoundIntent::Field.name(), "Field", "the term register's default");
    round_intent::set_intent(&mut w, HERO, RoundIntent::Field).expect("Field under the rule");
    assert_eq!(round_intent::target_problem(&w, HERO, RoundIntent::Field, RoundTarget::Cell(FOE_AT)), "", "a tile");
    assert_ne!(round_intent::target_problem(&w, HERO, RoundIntent::Field, RoundTarget::Unit(FOE)), "", "a click on a unit means the unit (IR19)");
    assert_ne!(round_intent::target_problem(&w, HERO, RoundIntent::Field, RoundTarget::Myself), "");
}

// ---- IR15, IR21: rows by derivation; the auto order holds primaries, the player's order any row ---------------------

#[test]
fn rows_are_derived_under_the_rule_and_legacy_without_it() {
    let on = session(field_on());
    let legacy = session(rules(&[ROUND_INTENT], json!({})));
    let rows = |w: &World, id: &str| round_intent::skill_rows(w, &w.cat.skills[sid(w, id)]);
    for id in ["f_burst", "f_arc", "f_line", "f_long"] {
        assert_eq!(rows(&on, id), ["field", "assault"], "{id}: dual, Field primary (IR15)");
        assert_eq!(rows(&legacy, id), ["assault"], "{id}: the legacy single intent (ATTACK's)");
    }
    assert_eq!(rows(&on, "strike"), ["assault"]);
    assert_eq!(rows(&on, "rest"), [ruleset::NEUTRAL]);
    let order = |w: &World, i: RoundIntent| rip::order(w, HERO, i).into_iter().map(|s| w.cat.skills[s].id.clone()).collect::<Vec<_>>();
    let field = order(&on, RoundIntent::Field);
    let assault = order(&on, RoundIntent::Assault);
    assert!(field.contains(&"f_burst".to_string()) && !assault.contains(&"f_burst".to_string()), "auto order: primary rows only ({field:?} / {assault:?})");
    assert!(assault.contains(&"strike".to_string()));
    // the player's order may hold the second row
    let mut w = on.clone();
    w.units[HERO].deck.order.insert("assault".into(), vec!["f_burst".into(), "strike".into()]);
    assert_eq!(order(&w, RoundIntent::Assault), ["f_burst", "strike"], "deck.order: any row");
    // legacy: the area attack is an Assault skill, no Field order
    assert!(order(&legacy, RoundIntent::Assault).contains(&"f_burst".to_string()));
}

#[test]
fn the_derived_field_bonus_by_skill() {
    let w = session(field_on());
    let bonus = |id: &str| skill::field_bonus(&w.cat.skills[sid(&w, id)]);
    assert_eq!(bonus("f_burst"), Some(FieldBonus::Reach));
    assert_eq!(bonus("f_arc"), Some(FieldBonus::Area));
    assert_eq!(bonus("f_line"), Some(FieldBonus::Area));
    assert_eq!(bonus("strike"), None, "no Field row");
    let hazards = session(rules(&[ROUND_INTENT, FIELD_INTENT, GROUND_HAZARDS], json!({})));
    assert_eq!(skill::field_bonus(&hazards.cat.skills[sid(&hazards, "f_pot")]), Some(FieldBonus::Lingering));
}

// ---- A4_EFFECTS section 2: the Field bonus sizes; the Assault row keeps the base size ------------------------------

#[test]
fn field_area_bonus_widens_arc_to_five_and_lengthens_line_to_four() {
    let w = session(field_on());
    let arc = &w.cat.skills[sid(&w, "f_arc")];
    let line = &w.cat.skills[sid(&w, "f_line")];
    let from = V2::new(4, 4);
    for source in [V2::new(5, 4), V2::new(4, 3), V2::new(3, 4), V2::new(4, 5)] {
        let base = skill::area_cells(arc, from, source);
        let wide = skill::field_cells(arc, from, source);
        assert_eq!((base.len(), wide.len()), (3, skill::FIELD_ARC_WIDTH as usize), "straight ARC toward {source}");
        assert!(base.iter().all(|c| wide.contains(c)), "the Field ARC holds the base ARC");
        let d = skill::snap(source - from);
        assert!(wide.iter().all(|c| (*c - source).x * d.x + (*c - source).y * d.y == 0), "5 across the direction, on the source's row");
        assert_eq!(skill::area_cells(line, from, source).len(), 3);
        assert_eq!(skill::field_cells(line, from, source), (0..skill::FIELD_LINE_LENGTH).map(|i| source + d * i).collect::<Vec<_>>(), "LINE 4 long");
    }
    // the exact diagonal: the L with arms of 2 (the base L's arms of 1)
    let diag = V2::new(5, 5);
    let base = skill::area_cells(arc, from, diag);
    let wide = skill::field_cells(arc, from, diag);
    assert_eq!((base.len(), wide.len()), (3, 5));
    assert!(base.iter().all(|c| wide.contains(c)));
    let thrown = &w.cat.skills[sid(&w, "f_burst")];
    let u = &w.units[HERO];
    let (lo, hi) = skill::reach_by(thrown, u, &w.ruleset);
    assert_eq!(skill::field_reach(thrown, u, &w.ruleset), (lo, hi + 1), "IR21 reach: +1");
    assert_eq!(skill::field_reach(arc, u, &w.ruleset), skill::reach_by(arc, u, &w.ruleset), "a drawn swing takes no reach bonus");
}

/// The Field row's `reach` bonus in the plan: a source one past the authored band is in reach in a Field round, out
/// of reach as a plain area aim of the legacy row.
#[test]
fn the_field_row_reaches_one_tile_further() {
    let mut w = session(field_on());
    let thrown = sid(&w, "f_burst");
    let (_, hi) = skill::reach_by(&w.cat.skills[thrown], &w.units[HERO], &w.ruleset);
    let far = HERO_AT + V2::new(hi + 1, 0);
    round_intent::set_intent(&mut w, HERO, RoundIntent::Field).unwrap();
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&w, HERO, PlanStep::area_at(thrown, far), true), "", "Field: band + 1");
    let legacy = session(rules(&[ROUND_INTENT], json!({})));
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&legacy, HERO, PlanStep::area_at(thrown, far), true), "Out of reach.");
}

// ---- IR4, IR10: one source tile a Field round; each family once ----------------------------------------------------

#[test]
fn a_field_round_has_one_source_tile_and_fills_each_family_once() {
    let mut w = session(field_on());
    w.units[HERO].deck.order.insert("field".into(), vec!["f_burst".into()]);
    let source = V2::new(4, 5);
    let (plan, r) = planned(&mut w, RoundIntent::Field, RoundTarget::Cell(source), None);
    assert_eq!(r, Ok(()));
    let bursts = entries_of(&w, &plan, "f_burst");
    assert_eq!(bursts.len(), 1, "field_fill_once: the family once, though blocks are left (IR10)");
    assert!(bursts.iter().all(|e| e.aim == source && e.target.is_none()), "fixed at planning on the source (IR4, IR5)");
    let blast = &w.cat.skills[sid(&w, "f_burst")];
    assert_eq!(bursts[0].blocks(&w), blast.blocks, "the Field row's wind-up is the authored one");
    let rest = sid(&w, "rest");
    assert!(plan.entries_for(&w, HERO).iter().any(|e| e.skill == rest), "Rest on the blocks left");
    // the impact tiles are the shape on the source, whoever stands there
    assert!(bursts[0].cells.contains(&FOE_AT) && bursts[0].cells.contains(&FOE2_AT), "{:?}", bursts[0].cells);
}

#[test]
fn a_field_source_out_of_band_brings_an_approach_and_a_directed_shape_a_neighbour() {
    let mut w = session(field_on());
    w.units[HERO].deck.order.insert("field".into(), vec!["f_arc".into()]);
    let source = V2::new(4, 3);
    let (plan, r) = planned(&mut w, RoundIntent::Field, RoundTarget::Cell(source), None);
    assert_eq!(r, Ok(()));
    assert!(plan.entries_for(&w, HERO).iter().any(|e| e.skill == sid(&w, "move")), "a standard approach first");
    let arcs = entries_of(&w, &plan, "f_arc");
    assert_eq!(arcs.len(), 1);
    let from = arcs[0].pos_before;
    assert!(from != source && (from.x - source.x).abs() <= 1 && (from.y - source.y).abs() <= 1, "swung from a neighbour of the source ({from})");
    assert_eq!(arcs[0].cells.len(), skill::FIELD_ARC_WIDTH as usize, "the Field ARC, 5 across");
}

// ---- IR17: the shaped Assault row ----------------------------------------------------------------------------------

#[test]
fn the_assault_row_locks_on_the_enemy_and_winds_up_longer() {
    let mut w = session(field_on());
    let (plan, r) = planned(&mut w, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_burst"));
    assert_eq!(r, Ok(()));
    let e = entries_of(&w, &plan, "f_burst").into_iter().next().expect("the Assault row's entry");
    let s = &w.cat.skills[sid(&w, "f_burst")];
    assert_eq!(e.target, Some(FOE), "target-locked (IR5: the target's tile at impact)");
    assert_eq!(e.blocks(&w), s.blocks + w.ruleset.numbers.assault_shape_blocks, "+assault_shape_blocks");
    assert_eq!(e.cells.len(), skill::area_cells(s, e.pos_before, FOE_AT).iter().filter(|c| **c != e.pos_before).count(), "base size, no Field bonus");
    let cmd = e.build_command(&w, HERO);
    assert_eq!(cmd.block_cost, e.blocks(&w));
    assert!(cmd.hazard.is_none());
    match cmd.kind {
        CmdKind::Area { target, splash, .. } => assert_eq!((target, splash), (Some(FOE), w.ruleset.numbers.splash_mult)),
        _ => panic!("an area command"),
    }
    // the legacy row (no field_intent): the area at the tile, the authored wind-up
    let mut legacy = session(rules(&[ROUND_INTENT], json!({})));
    let (plan, r) = planned(&mut legacy, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_burst"));
    assert_eq!(r, Ok(()));
    let e = entries_of(&legacy, &plan, "f_burst").into_iter().next().unwrap();
    assert_eq!((e.target, e.blocks(&legacy)), (None, s.blocks), "classic and round_intent alone: unchanged");
}

#[test]
fn the_assault_rows_extra_block_is_capped_by_the_round() {
    let mut w = session(field_on());
    let (plan, r) = planned(&mut w, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_long"));
    assert_eq!(r, Ok(()));
    let e = entries_of(&w, &plan, "f_long").into_iter().next().unwrap();
    assert_eq!(e.blocks(&w), BLOCKS, "capped at the round's blocks");
    for (n, want) in [(0, 2), (2, 4)] {
        let mut w = session(rules(&[ROUND_INTENT, FIELD_INTENT], json!({"assault_shape_blocks": n})));
        let (plan, _) = planned(&mut w, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_burst"));
        assert_eq!(entries_of(&w, &plan, "f_burst")[0].blocks(&w), want, "assault_shape_blocks {n}");
    }
}

/// IR5, IR17: the Assault row's shape is centred where its target stands at the impact; the Field row's tiles stay
/// where they were planned.
#[test]
fn the_assault_row_follows_its_target_and_the_field_row_does_not() {
    // a step that keeps the target in the band (2-3 from the hero), the second foe outside the moved shape
    let step_to = V2::new(4, 3);
    let mut w = session(field_on());
    let (plan, _) = planned(&mut w, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_burst"));
    let e = entries_of(&w, &plan, "f_burst").into_iter().next().unwrap();
    assert!(e.cells.contains(&FOE2_AT), "planned on the target's tile: the second foe inside");
    let cmd = e.build_command(&w, HERO);
    let done = resolve(&mut w, cmd, |w| w.grid.positions[FOE] = Some(step_to));
    assert!(w.units[FOE].hp < HP, "the target is hit where it went");
    assert_eq!(w.units[FOE2].hp, HP, "the planned tiles are not the impact's: the second foe is out of the moved shape");
    if let CmdKind::Area { cells, .. } = done.kind {
        assert!(cells.contains(&step_to), "re-read on the target's tile");
    }
    let mut w = session(field_on());
    let (plan, _) = planned(&mut w, RoundIntent::Field, RoundTarget::Cell(FOE_AT), Some("f_burst"));
    let e = entries_of(&w, &plan, "f_burst").into_iter().next().unwrap();
    let cmd = e.build_command(&w, HERO);
    resolve(&mut w, cmd, |w| w.grid.positions[FOE] = Some(V2::new(8, 1)));
    assert_eq!(w.units[FOE].hp, HP, "the Field row's marks stay: the foe walked out");
    assert!(w.units[FOE2].hp < HP, "the foe left in the marks is hit");
}

/// IR17: `splash_mult` off the target, the status on the target only.
#[test]
fn the_assault_row_splashes_off_its_target_and_lands_its_status_on_it_only() {
    let hit = |splash: f64| {
        let mut w = session(rules(&[ROUND_INTENT, FIELD_INTENT], json!({"splash_mult": splash})));
        let (plan, _) = planned(&mut w, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_burst"));
        let cmd = entries_of(&w, &plan, "f_burst")[0].build_command(&w, HERO);
        resolve(&mut w, cmd, |_| {});
        (HP - w.units[FOE].hp, HP - w.units[FOE2].hp, w.units[FOE2].st.has_any("weaken"))
    };
    let (full_target, full_side, _) = hit(1.0);
    let (target, side, side_status) = hit(ruleset::SPLASH_MULT);
    assert!(full_side > 0 && side > 0, "both foes in the shape");
    assert_eq!(target, full_target, "the target takes the full power");
    assert!(side < full_side, "the off-target hit takes splash_mult ({side} < {full_side})");
    assert!(!side_status, "the status lands on the target only");
}

/// IR9's test under the rule: an ally beside the target is unhurt by the Assault row and hit by the Field row when it
/// stands in the marks.
#[test]
fn an_ally_beside_the_target_is_spared_by_the_assault_row_and_hit_by_the_field_row() {
    let run = |intent: RoundIntent, target: RoundTarget| {
        let mut w = session(field_on());
        w.grid.positions[ALLY] = Some(FOE2_AT);
        w.grid.positions[FOE2] = Some(V2::new(9, 0));
        let (plan, r) = planned(&mut w, intent, target, Some("f_burst"));
        assert_eq!(r, Ok(()), "{intent:?}");
        let cmd = entries_of(&w, &plan, "f_burst")[0].build_command(&w, HERO);
        resolve(&mut w, cmd, |_| {});
        (w.units[ALLY].hp < HP, w.units[FOE].hp < HP)
    };
    assert_eq!(run(RoundIntent::Assault, RoundTarget::Unit(FOE)), (false, true), "Assault: foes only");
    assert_eq!(run(RoundIntent::Field, RoundTarget::Cell(FOE_AT)), (true, true), "Field: all in the marks");
}

#[test]
fn the_field_rows_hazard_lingers_one_round_more() {
    let rows = [ROUND_INTENT, FIELD_INTENT, GROUND_HAZARDS];
    let mut w = session(rules(&rows, json!({})));
    let (plan, r) = planned(&mut w, RoundIntent::Field, RoundTarget::Cell(FOE_AT), Some("f_pot"));
    assert_eq!(r, Ok(()));
    let cmd = entries_of(&w, &plan, "f_pot")[0].build_command(&w, HERO);
    let duration = w.cat.hazards.get("f_fire").unwrap().duration;
    assert_eq!(cmd.hazard.as_ref().map(|p| p.rounds), Some((duration + 1).min(rpgkit_battle::ground::HAZARD_ROUNDS_CAP)), "lingering: +1 round");
    // without field_intent the hazard keeps the row's duration
    let mut legacy = session(rules(&[ROUND_INTENT, GROUND_HAZARDS], json!({})));
    let (plan, _) = planned(&mut legacy, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_pot"));
    let cmd = entries_of(&legacy, &plan, "f_pot")[0].build_command(&legacy, HERO);
    assert_eq!(cmd.hazard.as_ref().map(|p| p.rounds), Some(0), "0 = the row's duration");
}

// ---- IR19: the first pick's target picks the row --------------------------------------------------------------------

#[test]
fn a_dual_skill_picked_on_a_tile_is_field_and_on_an_enemy_assault() {
    let w = session(field_on());
    let burst = vec![Pick::Skill { skill: sid(&w, "f_burst"), kind: None }];
    assert_eq!(rip::pick_intent(&w, HERO, RoundTarget::Cell(FOE_AT), &burst), Some(RoundIntent::Field));
    assert_eq!(rip::pick_intent(&w, HERO, RoundTarget::Unit(FOE), &burst), Some(RoundIntent::Assault));
    let legacy = session(rules(&[ROUND_INTENT], json!({})));
    assert_eq!(rip::pick_intent(&legacy, HERO, RoundTarget::Cell(FOE_AT), &burst), Some(RoundIntent::Assault), "legacy: the single intent");
    // a whole pick on a tile plans the Field row
    let mut w = w.clone();
    let mut plan = Plan::new();
    let (r, _) = rip::pick(&mut w, &mut plan, HERO, RoundTarget::Cell(FOE_AT), &burst);
    assert_eq!(r, Ok(()));
    assert_eq!(round_intent::intent_of(&w, HERO), Some(RoundIntent::Field));
    assert!(entries_of(&w, &plan, "f_burst").iter().all(|e| e.target.is_none() && e.aim == FOE_AT));
}

// ---- AI (minimal row Z): a cluster gets a Field round, a lone foe the Assault row -----------------------------------

#[test]
fn the_ai_takes_a_field_round_on_a_cluster_and_alternates() {
    let mut w = session(field_on());
    let profile = w.profile_of(HERO);
    assert!(profile.area_threshold() <= 2, "two foes are a cluster for this profile");
    let mut plan = Plan::new();
    let round = ai::plan_round_intent(&mut w, &mut plan, HERO, &profile, Tactic::Balanced);
    assert_eq!(round.row, ai::ROW_FIELD, "two foes inside one source");
    assert_eq!(round.pick.as_ref().map(|p| p.intent), Some(RoundIntent::Field));
    // held Field last round: it alternates
    let mut plan = Plan::new();
    let round = ai::plan_round_intent(&mut w, &mut plan, HERO, &profile, Tactic::Balanced);
    assert_ne!(round.row, ai::ROW_FIELD, "not two Field rounds in a row");
    // a lone foe: no Field round
    let mut w = session(field_on());
    w.grid.positions[FOE2] = Some(V2::new(9, 0));
    let mut plan = Plan::new();
    assert_ne!(ai::plan_round_intent(&mut w, &mut plan, HERO, &profile, Tactic::Balanced).row, ai::ROW_FIELD);
    // without the rule the table is as before
    let mut legacy = session(rules(&[ROUND_INTENT], json!({})));
    let mut plan = Plan::new();
    assert_ne!(ai::plan_round_intent(&mut legacy, &mut plan, HERO, &profile, Tactic::Balanced).row, ai::ROW_FIELD);
}

// ---- Review fixes: IR5/IR22 Stand rings, IR2 Assault on a tile, chase and miss, splash factor, IR15 self is Stand -----

/// IR5, IR6, IR22: a RING (Stand only under IR21) plans in a Stand (`guard`) round anchored on the user's tile and hits
/// the enemies around it (Stand: enemies only); with no enemy around the ring is not cast.
#[test]
fn a_ring_in_a_stand_round_is_self_anchored_and_hits_the_enemies_around() {
    let mut w = session(field_on());
    assert_eq!(round_intent::skill_rows(&w, &w.cat.skills[sid(&w, "f_ring")]), ["guard"]);
    w.grid.positions[FOE] = Some(HERO_AT + V2::new(1, 1));
    w.grid.positions[ALLY] = Some(HERO_AT + V2::new(-1, 0));
    w.units[HERO].deck.order.insert("guard".into(), vec!["f_ring".into()]);
    let (plan, r) = planned(&mut w, RoundIntent::Guard, RoundTarget::Myself, None);
    assert_eq!(r, Ok(()));
    let rings = entries_of(&w, &plan, "f_ring");
    assert!(!rings.is_empty());
    for e in &rings {
        assert_eq!(e.cells, skill::ring_cells(HERO_AT), "the 8 tiles around the user");
    }
    let cmd = rings[0].build_command(&w, HERO);
    resolve(&mut w, cmd, |_| {});
    assert!(w.units[FOE].hp < HP, "the adjacent enemy is hit");
    assert_eq!(w.units[ALLY].hp, HP, "Stand: enemies only (IR9)");
    // nobody around: the ring is not the opener
    let mut w = session(field_on());
    w.units[HERO].deck.order.insert("guard".into(), vec!["f_ring".into()]);
    let (_, r) = planned(&mut w, RoundIntent::Guard, RoundTarget::Myself, None);
    assert!(r.is_err(), "an empty ring is not cast: {r:?}");
    // the AI's Guard rows open with the Defend skill, not the ring
    let w = session(field_on());
    let mut w2 = w.clone();
    w2.grid.positions[FOE] = Some(HERO_AT + V2::new(1, 0));
    w2.units[HERO].hp = HP / 10;
    let mut profile = w2.profile_of(HERO);
    profile.flee_ratio = 0.0;
    let mut plan = Plan::new();
    let round = ai::plan_round_intent(&mut w2, &mut plan, HERO, &profile, Tactic::Balanced);
    assert_eq!((round.row, round.pick.as_ref().map(|p| p.intent)), (ai::ROW_GUARD, Some(RoundIntent::Guard)), "hurt: AI row 3");
    let first = plan.entries_for(&w2, HERO).into_iter().find(|e| e.step_index >= 0).map(|e| w2.cat.skills[e.skill].id.clone());
    assert_eq!(first.as_deref(), Some("guard"), "the Defend opens; the ring may follow");
}

/// IR2, IR19: under the rule a tile is Field's, an enemy Assault's: Assault on a tile is refused (the shaped Assault
/// row always locks on a unit) and a dual skill picked on a tile plans its Field row. Without the rule Assault still
/// takes a tile.
#[test]
fn assault_on_a_tile_is_refused_under_the_rule_and_the_tile_goes_to_field() {
    let mut w = session(field_on());
    assert_ne!(round_intent::target_problem(&w, HERO, RoundIntent::Assault, RoundTarget::Cell(FOE_AT)), "");
    round_intent::set_intent(&mut w, HERO, RoundIntent::Assault).unwrap();
    assert!(round_intent::set_target(&mut w, HERO, RoundTarget::Cell(FOE_AT)).is_err(), "no Assault round on a tile");
    let burst = vec![Pick::Skill { skill: sid(&w, "f_burst"), kind: None }];
    let mut plan = Plan::new();
    let (r, _) = rip::pick(&mut w, &mut plan, HERO, RoundTarget::Cell(FOE_AT), &burst);
    assert_eq!(r, Ok(()));
    let e = entries_of(&w, &plan, "f_burst").into_iter().next().unwrap();
    assert_eq!((round_intent::intent_of(&w, HERO), e.target), (Some(RoundIntent::Field), None), "the tile pick is the Field row");
    let legacy = session(rules(&[ROUND_INTENT], json!({})));
    assert_eq!(round_intent::target_problem(&legacy, HERO, RoundIntent::Assault, RoundTarget::Cell(FOE_AT)), "", "legacy: unchanged");
}

/// HOOKS1 AT2 `flush_cover` under the rule: its aim at a tile is a Field round (with the Field order and tiles).
#[test]
fn flush_cover_aims_a_field_round_under_the_rule() {
    let play = |list: &[&str]| {
        let mut w = session(rules(list, json!({})));
        // an aim at (4, 5), in both bands, catches all three; an aim at the first ranked foe's tile two
        w.grid.positions[FOE3] = Some(V2::new(4, 6));
        w.profiles.get_mut(&HERO).expect("profile").tactics.push(UnitTactic::FlushCover);
        if round_intent::field_rows(&w) {
            // held Field last round: row Z waits, the Assault row's flush_cover decides
            round_intent::set_intent(&mut w, HERO, RoundIntent::Field).unwrap();
        }
        let profile = w.profile_of(HERO);
        let mut plan = Plan::new();
        let round = ai::plan_round_intent(&mut w, &mut plan, HERO, &profile, Tactic::Balanced);
        (round.row, round.pick.map(|p| (p.intent, p.target)))
    };
    let (row, pick) = play(&[ROUND_INTENT, FIELD_INTENT, AI_TACTICS]);
    assert_eq!(row, ai::ROW_ASSAULT);
    assert!(matches!(pick, Some((RoundIntent::Field, RoundTarget::Cell(_)))), "{pick:?}");
    let (_, pick) = play(&[ROUND_INTENT, AI_TACTICS]);
    assert!(matches!(pick, Some((RoundIntent::Assault, RoundTarget::Cell(_)))), "legacy: an Assault round on the tile ({pick:?})");
}

/// The Assault row's command on FOE, planned from the hero's tile.
fn assault_cmd(w: &mut World) -> Command {
    let (plan, r) = planned(w, RoundIntent::Assault, RoundTarget::Unit(FOE), Some("f_burst"));
    assert_eq!(r, Ok(()));
    entries_of(w, &plan, "f_burst")[0].build_command(w, HERO)
}

/// IR17 (MR16): a target out of range as the command starts is chased one tile, then hit; one that leaves the range
/// after the start, dies or hides before the impact is missed (the shape finds no tiles: no one else is hit either).
#[test]
fn the_assault_row_chases_at_its_start_and_misses_a_target_gone_dead_or_hidden() {
    let mut w = session(field_on());
    let cmd = assault_cmd(&mut w);
    let (_, hi) = skill::reach_by(&w.cat.skills[sid(&w, "f_burst")], &w.units[HERO], &w.ruleset);
    w.grid.positions[FOE] = Some(HERO_AT + V2::new(hi + 1, 0));
    w.grid.positions[FOE2] = Some(V2::new(9, 0));
    let done = resolve(&mut w, cmd, |_| {});
    assert_eq!(done.moved, Some(rpgkit_battle::command::MoveKind::Chase), "{}", done.log);
    assert_ne!(w.pos(HERO), HERO_AT, "one tile closer");
    assert!(w.units[FOE].hp < HP, "then hit");
    type Gone = fn(&mut World);
    let cases: [(&str, Gone); 3] = [
        ("left the range", |w: &mut World| w.grid.positions[FOE] = Some(V2::new(9, 4))),
        ("died", |w: &mut World| w.units[FOE].hp = 0),
        ("hid", |w: &mut World| {
            let spec = rpgkit_rules::status::EffectSpec::grant("stealth", 2, 0, "test");
            rpgkit_rules::status::apply_effect(&mut w.units[FOE], &spec, rpgkit_rules::status::Timing::Now, 0.0, None);
            w.round_states.entry(FOE).or_default().moved = true;
        }),
    ];
    for (why, gone) in cases {
        let mut w = session(field_on());
        let cmd = assault_cmd(&mut w);
        let before = w.units[FOE].hp;
        let done = resolve(&mut w, cmd, gone);
        assert!(done.log.starts_with("[Area Miss]"), "{why}: {}", done.log);
        assert_eq!(w.units[FOE2].hp, HP, "{why}: the foe beside is not hit either");
        if why != "died" {
            assert_eq!(w.units[FOE].hp, before, "{why}");
        }
    }
}

/// IR17: the off-target hit is the full hit times `splash_mult` (up to rounding), for each value of its range.
#[test]
fn the_splash_factor_is_splash_mult() {
    let side = |splash: f64| {
        let mut w = session(rules(&[ROUND_INTENT, FIELD_INTENT], json!({"splash_mult": splash})));
        let cmd = assault_cmd(&mut w);
        resolve(&mut w, cmd, |_| {});
        (HP - w.units[FOE].hp, HP - w.units[FOE2].hp)
    };
    let (target, full) = side(1.0);
    assert!(full > 4, "a hit large enough to read a factor ({full})");
    for splash in [0.25, ruleset::SPLASH_MULT, 0.75] {
        let (t, s) = side(splash);
        assert_eq!(t, target, "the target's hit does not change");
        assert!((s as f64 - full as f64 * splash).abs() <= 1.0, "splash {splash}: {s} vs {full} x {splash}");
    }
}

/// IR3 (b), IR15: a self-or-ally Support skill picked on yourself plans in Stand (`guard`); on an ally in Support.
#[test]
fn a_self_target_is_always_stand() {
    let mut cat = catalog();
    let heal = rpgkit_data::SkillDef::from_obj(json!({"id": "f_mend", "name": "f_mend", "kind": "SUPPORT", "level": 1, "blocks": 1, "heal_base": 10}).as_object().unwrap());
    cat.skills.push(heal);
    let row = json!({"id": "f_fight", "name": "Fight", "party": ["p_hero", "p_ally"], "enemies": ["p_foe"], "total_blocks": BLOCKS, "unit_ap": 10, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = field_on();
    let s = Session::start(Arc::new(cat), def, &IndexMap::new());
    let mut w = s.w.clone();
    w.grid.positions[ALLY] = Some(HERO_AT + V2::new(0, 1));
    let mend = vec![Pick::Skill { skill: sid(&w, "f_mend"), kind: None }];
    assert_eq!(round_intent::skill_rows(&w, &w.cat.skills[sid(&w, "f_mend")]), ["support", "guard"]);
    assert_eq!(rip::pick_intent(&w, HERO, RoundTarget::Myself, &mend), Some(RoundIntent::Guard), "self: Stand");
    assert_eq!(rip::pick_intent(&w, HERO, RoundTarget::Unit(ALLY), &mend), Some(RoundIntent::Support), "an ally: Support");
    let mut plan = Plan::new();
    let (r, _) = rip::pick(&mut w, &mut plan, HERO, RoundTarget::Myself, &mend);
    assert_eq!(r, Ok(()));
    assert_eq!(round_intent::intent_of(&w, HERO), Some(RoundIntent::Guard));
    assert_eq!(entries_of(&w, &plan, "f_mend").len(), 1, "the heal is planned in the Stand round");
}

/// IR15 under the rule, kept for items until K3 gives them their rows: a self or ally item used on yourself still plans
/// as Support on Myself (an item has its one legacy intent; only a skill clicked on itself takes Stand).
#[test]
fn an_item_used_on_yourself_stays_support_until_items_get_their_rows() {
    let mut w = session(field_on());
    let potion = rpgkit_data::ItemDef::from_obj(json!({"id": "f_potion", "name": "Potion", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 20}).as_object().unwrap());
    assert!(w.units[HERO].inventory.add_to_pouch(&potion));
    w.add_item_skills();
    let use_it = vec![Pick::Item("f_potion".into())];
    assert_eq!(rip::pick_intent(&w, HERO, RoundTarget::Myself, &use_it), Some(RoundIntent::Support));
    let mut plan = Plan::new();
    let (r, _) = rip::pick(&mut w, &mut plan, HERO, RoundTarget::Myself, &use_it);
    assert_eq!(r, Ok(()));
    assert_eq!((round_intent::intent_of(&w, HERO), w.round_states[&HERO].target), (Some(RoundIntent::Support), Some(RoundTarget::Myself)));
    assert!(plan.entries_for(&w, HERO).iter().any(|e| e.item.as_ref().is_some_and(|i| i.id == "f_potion")), "the potion is planned");
}

/// FB-120 (IR20 table): a Field row with the derived `power` bonus hits at `FIELD_POWER_MULT` x the skill's power; the
/// bonus sizes stay constants. Without the rule the same skill hits at its power.
#[test]
fn the_field_power_bonus_multiplies_the_hit() {
    let w = session(field_on());
    let near = &w.cat.skills[sid(&w, "f_near")];
    assert_eq!(round_intent::skill_rows(&w, near), ["field"], "a reach-1 BURST: Field only");
    assert_eq!(skill::field_bonus(near), Some(FieldBonus::Power));
    assert_eq!((skill::FIELD_POWER_MULT, skill::field_power(near)), (1.25, 1.25));
    assert_eq!(skill::field_power(&w.cat.skills[sid(&w, "f_burst")]), 1.0, "a reach bonus is not a power bonus");
    let mult = |rules: Ruleset, intent: RoundIntent| {
        let mut w = session(rules);
        w.grid.positions[FOE] = Some(HERO_AT + V2::new(1, 0));
        let target = if intent == RoundIntent::Field { RoundTarget::Cell(HERO_AT + V2::new(1, 0)) } else { RoundTarget::Unit(FOE) };
        let (plan, r) = planned(&mut w, intent, target, Some("f_near"));
        assert_eq!(r, Ok(()), "{intent:?}");
        let e = entries_of(&w, &plan, "f_near")[0].clone();
        let cmd = e.build_command(&w, HERO);
        let base = w.units[HERO].power_for(&w.cat.skills[sid(&w, "f_near")]);
        match cmd.kind {
            CmdKind::Area { skill_mult, .. } => skill_mult / base,
            _ => panic!("an area"),
        }
    };
    assert!((mult(field_on(), RoundIntent::Field) - skill::FIELD_POWER_MULT).abs() < 1e-9, "Field: x1.25");
    assert!((mult(rules(&[ROUND_INTENT], json!({})), RoundIntent::Assault) - 1.0).abs() < 1e-9, "without the rule: x1");
}
