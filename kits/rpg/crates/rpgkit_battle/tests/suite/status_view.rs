//! STATUSVIS1-B (PR7-57, `docs/specs/STATUSVIS1.md` SV5-SV15) on a scratch catalog built here (no shipped data): the
//! kit reads that show a status where it acts (`status_view`). Expectations come from the status table and the rules:
//! the locked runs from each row's `lock_blocks`, the plate priority from its primitives, the ticks from
//! `status::tick_amount` in the resolver's order, one planned loss from the foe card's hits plus the ticks, a hit's
//! unmodified number from the same forecast with the modifier taken out of the world, the counter from the counter's
//! own multiplier, and a row's one target from the first action that aims at one.

use indexmap::IndexMap;
use rpgkit_battle::grid::V2;
use rpgkit_battle::intent;
use rpgkit_battle::plan::{Plan, PlanStep};
use rpgkit_battle::resolver::ScheduledAction;
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_battle::status_view::{self as sv, RowTarget};
use rpgkit_battle::world::{UnitId, World};
use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::ruleset::{Ruleset, TIMELINE};
use rpgkit_rules::status::{self, Primitive};
use serde_json::json;
use std::sync::Arc;

const HERO: UnitId = 0;
const ALLY: UnitId = 1;
const FOE: UnitId = 2;

/// The hero next to the foe (west), the ally behind the hero, under the timeline.
fn session() -> Session {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 1, "basic": true, "mitigation": 0.3}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
        json!({"id": "c_hit", "name": "Hit", "kind": "ATTACK", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 1.0, "target_mode": "ENEMY"}),
        json!({"id": "c_arc", "name": "Arc", "kind": "ATTACK", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 1.0, "area": "ARC", "affects": "all", "target_mode": "ENEMY"}),
        json!({"id": "c_mend", "name": "Mend", "kind": "SUPPORT", "round_intent": "support", "level": 1, "blocks": 2, "ap_cost": 2, "heal_base": 30, "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF"}),
    ];
    let unit = |id: &str, side: &str, speed: i32| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 500, "power": 100, "armor": 10, "speed": speed, "attack_range": 1, "skill_cap": 5});
    let cat = Catalog::from_value(&json!({"skills": skills, "units": [unit("c_hero", "party", 14), unit("c_ally", "party", 12), unit("c_foe", "enemy", 10)]}));
    let row = json!({"id": "c_fight", "name": "Fight", "party": ["c_hero", "c_ally"], "enemies": ["c_foe"], "total_blocks": 6, "unit_ap": 12, "ap_regen": 0, "max_rounds": 8, "rng_seed": 1337});
    let mut def = EncounterDef::from_obj(row.as_object().unwrap());
    def.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": []}).as_object().unwrap());
    let mut s = Session::start(Arc::new(cat), def, &IndexMap::new());
    for (c, cell) in [(HERO, V2::new(6, 4)), (ALLY, V2::new(5, 4)), (FOE, V2::new(7, 4))] {
        s.w.grid.positions[c] = Some(cell);
    }
    s.resolver.scheduled.clear();
    for c in [HERO, ALLY, FOE] {
        s.resolver.register(c);
    }
    s
}

fn sid(w: &World, id: &str) -> usize {
    w.book().find(id).unwrap_or_else(|| panic!("skill {id}"))
}

/// The round as it will run with the hero hitting the foe (one Hit).
fn hit_round(w: &World) -> Vec<ScheduledAction> {
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, HERO, PlanStep::engage(sid(w, "c_hit"), FOE), true), "", "the hero hits the foe");
    intent::planned_schedule(w, &[], &plan)
}

fn give(w: &mut World, u: UnitId, st: &str, magnitude: i32) {
    status::grant_now(&mut w.units[u], st, 2, magnitude, "test");
}

/// The runs a status's own `lock_blocks` make: blocks in a row join, blocks apart stand alone.
fn runs_of(st: &str) -> Vec<(i32, i32)> {
    let mut out: Vec<(i32, i32)> = Vec::new();
    let mut blocks: Vec<i32> = status::blocked_blocks(st).to_vec();
    blocks.sort();
    for b in blocks {
        match out.last_mut() {
            Some(r) if r.1 + 1 == b => r.1 = b,
            _ => out.push((b, b)),
        }
    }
    out
}

#[test]
fn a_lock_shows_as_one_run_per_stretch_of_its_blocks() {
    for st in ["stun", "stagger"] {
        let mut s = session();
        give(&mut s.w, HERO, st, 0);
        let runs: Vec<(i32, i32)> = sv::lock_runs(&s.w, HERO).iter().map(|r| (r.start, r.end)).collect();
        assert_eq!(runs, runs_of(st), "{st}: its lock_blocks {:?}", status::blocked_blocks(st));
        assert!(sv::lock_runs(&s.w, HERO).iter().all(|r| r.status == st));
        // the plan books exactly these blocks as lost
        let lost: Vec<i32> = Plan::new().compile(&s.w, HERO).entries.iter().filter(|e| !e.status_name.is_empty()).map(|e| e.start_block).collect();
        let shown: Vec<i32> = runs.iter().flat_map(|r| r.0..=r.1).collect();
        assert_eq!(lost, shown, "{st}: the runs cover the plan's lost blocks");
    }
    // SV5, SV15 as the table has them today: stun one continuous run, stagger two single blocks
    assert_eq!(runs_of("stun").len(), 1);
    assert_eq!(runs_of("stagger").len(), 2);
    let s = session();
    assert!(sv::lock_runs(&s.w, HERO).is_empty(), "no lock, no run");
}

#[test]
fn the_plate_priority_follows_the_primitives() {
    for r in status::ROWS.iter() {
        let want = if r.has_primitive(Primitive::Reactive) || r.has_primitive(Primitive::RuleFlag) {
            1
        } else if r.has_primitive(Primitive::Tick) {
            2
        } else {
            3
        };
        assert_eq!(sv::plate_priority(r.id), want, "{}", r.id);
    }
    assert_eq!(sv::plate_priority("no_such_status"), 3);
}

#[test]
fn ticks_join_the_hits_in_one_planned_loss() {
    let mut s = session();
    give(&mut s.w, FOE, "bleed", 0);
    let sched = hit_round(&s.w);
    let card = intent::foe_card(&s.w, &sched, FOE);
    assert!(card.total(None) > 0, "the hit lands");
    let bleed = status::tick_amount(&s.w.units[FOE], status::row_of("bleed").unwrap(), &s.w.ruleset.numbers);
    assert!(bleed > 0);
    let ticks = sv::round_ticks(&s.w, FOE);
    assert_eq!(ticks, vec![sv::Tick { status: "bleed".into(), hp: -bleed }]);
    let pl = sv::planned_loss(&s.w, &sched, FOE);
    assert_eq!(pl.hit_hp(), card.total(None), "the hits are the foe card's");
    assert_eq!(pl.loss, card.total(None) + bleed, "one loss: the hits and the tick");
    assert_eq!(pl.gain, 0);
    // within the HP left
    s.w.units[FOE].hp = 3;
    assert_eq!(sv::planned_loss(&s.w, &sched, FOE).loss, 3);
}

#[test]
fn a_healing_tick_is_a_gain_within_the_missing_hp() {
    let mut s = session();
    give(&mut s.w, ALLY, "regen", 10);
    assert!(sv::round_ticks(&s.w, ALLY).is_empty(), "at full HP the tick heals nothing");
    s.w.units[ALLY].hp -= 7;
    let amount = status::tick_amount(&s.w.units[ALLY], status::row_of("regen").unwrap(), &s.w.ruleset.numbers);
    let ticks = sv::round_ticks(&s.w, ALLY);
    assert_eq!(ticks, vec![sv::Tick { status: "regen".into(), hp: amount.min(7) }]);
    assert_eq!(sv::planned_loss(&s.w, &[], ALLY).gain, amount.min(7));
}

#[test]
fn a_hit_shows_its_number_after_modifiers_with_the_unmodified_base() {
    let plain = session();
    let base_dmg = sv::loss_reads(&plain.w, &hit_round(&plain.w), FOE)[0].parts.damage;
    let read = &sv::loss_reads(&plain.w, &hit_round(&plain.w), FOE)[0];
    assert_eq!(read.parts.base, read.parts.damage, "no modifier: the base is the number");
    assert!(read.parts.mods.is_empty());
    assert_eq!(read.parts.sign(), 0);

    // the attacker's empower raises it; its base is the hit without the status
    let mut s = session();
    give(&mut s.w, HERO, "empower", 30);
    let sched = hit_round(&s.w);
    let parts = sv::loss_reads(&s.w, &sched, FOE)[0].parts.clone();
    assert_eq!(parts.damage, intent::foe_card(&s.w, &sched, FOE).hits[0].damage, "the number is the forecast's");
    assert!(parts.damage > base_dmg, "empowered: {} over {}", parts.damage, base_dmg);
    assert_eq!(parts.base, base_dmg, "the base takes the empower out");
    assert_eq!(parts.sign(), 1);
    assert_eq!(parts.mods.len(), 1);
    assert_eq!(parts.mods[0].source, "empower");
    assert!(parts.mods[0].pct > 0);

    // the target's expose raises it too; weaken lowers it
    let mut s = session();
    give(&mut s.w, FOE, "expose", 10);
    let sched = hit_round(&s.w);
    let parts = sv::loss_reads(&s.w, &sched, FOE)[0].parts.clone();
    assert_eq!(parts.base, base_dmg, "the base takes the expose out");
    assert!(parts.damage >= base_dmg);
    let mut s = session();
    give(&mut s.w, HERO, "weaken", 25);
    let sched = hit_round(&s.w);
    let parts = sv::loss_reads(&s.w, &sched, FOE)[0].parts.clone();
    assert!(parts.damage < base_dmg, "weakened: {} under {}", parts.damage, base_dmg);
    assert_eq!(parts.base, base_dmg);
    assert_eq!(parts.sign(), -1);
    assert!(parts.mods.iter().any(|m| m.source == "weaken" && m.pct < 0));
}

#[test]
fn a_riposte_draws_a_counter_from_a_melee_hit_only() {
    let mut s = session();
    let sched = hit_round(&s.w);
    assert_eq!(sv::loss_reads(&s.w, &sched, FOE)[0].counter, None, "no riposte, no counter");
    give(&mut s.w, FOE, "riposte", 0);
    let sched = hit_round(&s.w);
    let d = &s.w.units[FOE];
    let want = damage::compute(d, &s.w.units[HERO], HitInput { skill_mult: s.w.ruleset.numbers.counter_mult * d.weaken_factor() * d.empower_factor(), ..HitInput::default() }).final_dmg;
    assert_eq!(sv::loss_reads(&s.w, &sched, FOE)[0].counter, Some(want));
    assert_eq!(sv::counter_return(&s.w, FOE, HERO), Some(want));
    // a ranged attacker draws none
    s.w.units[HERO].attack_range = 3;
    let a = sched.iter().find(|a| a.unit == HERO).expect("the hit");
    assert_eq!(sv::counter_on(&s.w, &sched, a, FOE), None);
}

/// FB-370: the counter follows the riposte rule (`command::riposte_answers_at`), not the skill kind: a melee area
/// skill's hit draws it; a ranged hit, a flank (rule `flank`) and a hit from a tile not adjacent draw none; an attacker
/// draws it once per round however many hits it plans.
#[test]
fn the_counter_follows_the_riposte_rule_not_the_skill_kind() {
    use rpgkit_battle::command::CmdKind;
    use rpgkit_battle::round_intent::{self, RoundIntent, RoundTarget};
    use rpgkit_rules::ruleset::{FLANK, ROUND_INTENT};
    let mut s = session();
    give(&mut s.w, FOE, "riposte", 0);
    let want = sv::counter_return(&s.w, FOE, HERO).expect("the riposte's return");
    // a melee area skill (an Attack with an ARC area) draws it
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "c_arc"), FOE), true), "", "the hero sweeps the foe");
    let sched = intent::planned_schedule(&s.w, &[], &plan);
    let a = sched.iter().find(|a| a.unit == HERO && matches!(a.command.kind, CmdKind::Area { .. })).expect("an area hit");
    assert!(intent::hit_on(&s.w, &sched, a, FOE).is_some(), "the arc strikes the foe");
    assert_eq!(sv::counter_on(&s.w, &sched, a, FOE), Some(want), "a melee area hit draws the counter");
    assert_eq!(sv::loss_reads(&s.w, &sched, FOE)[0].counter, Some(want));
    // once per attacker: two hits in the round, the counter on the first only
    let mut plan = Plan::new();
    for _ in 0..2 {
        assert_eq!(plan.add_step(&s.w, HERO, PlanStep::engage(sid(&s.w, "c_hit"), FOE), true), "", "the hero hits twice");
    }
    let sched = intent::planned_schedule(&s.w, &[], &plan);
    let reads = sv::loss_reads(&s.w, &sched, FOE);
    assert_eq!(reads.len(), 2, "two hits");
    assert_eq!(reads.iter().map(|r| r.counter).collect::<Vec<_>>(), vec![Some(want), None], "answered once");
    // ranged: none
    let sched = hit_round(&s.w);
    let a = sched.iter().find(|a| a.unit == HERO).expect("the hit").clone();
    let mut ranged = s.w.clone();
    ranged.units[HERO].attack_range = 3;
    assert_eq!(sv::counter_on(&ranged, &sched, &a, FOE), None, "a ranged hit");
    // not adjacent at the impact (the foe two tiles off): none
    let mut far = s.w.clone();
    far.grid.positions[FOE] = Some(V2::new(9, 4));
    assert_eq!(sv::counter_on(&far, &sched, &a, FOE), None, "not adjacent");
    // a flank (the foe engaged with the ally on its far side, the hero behind it): none under the rule, the counter
    // without it
    for (rules, draws) in [(vec![ROUND_INTENT, FLANK], false), (vec![ROUND_INTENT], true)] {
        let mut w = s.w.clone();
        w.ruleset = Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap());
        w.grid.positions[ALLY] = Some(V2::new(8, 4));
        round_intent::set_intent(&mut w, FOE, RoundIntent::Assault).expect("intent");
        round_intent::set_target(&mut w, FOE, RoundTarget::Unit(ALLY)).expect("target");
        assert_eq!(round_intent::flank_bonus_from(&w, HERO, V2::new(6, 4), FOE), !draws, "{rules:?}: the flank");
        assert_eq!(sv::counter_on(&w, &sched, &a, FOE).is_some(), draws, "{rules:?}");
    }
}

#[test]
fn a_row_names_its_one_target_for_the_round() {
    let s = session();
    let w = &s.w;
    assert_eq!(sv::row_aim(w, &hit_round(w), HERO).target, RowTarget::Foe(FOE));
    assert!(!sv::row_aim(w, &hit_round(w), HERO).area);
    let mut plan = Plan::new();
    assert_eq!(plan.add_step(w, ALLY, PlanStep::engage(sid(w, "c_mend"), HERO), true), "", "the ally mends the hero");
    let sched = intent::planned_schedule(w, &[], &plan);
    assert_eq!(sv::row_aim(w, &sched, ALLY).target, RowTarget::Ally(HERO));
    assert_eq!(sv::row_aim(w, &sched, HERO).target, RowTarget::None, "nothing aimed: a dash");
}
