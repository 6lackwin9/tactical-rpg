//! The damage pipeline (U1.2 slice 1, `FB-05`) on a scratch catalog (no shipped data). The stages run in their fixed
//! order whatever order the modifiers come in; adds sit in base and flat, the base stage may round; `damage::compute`
//! is an exact port of the formula `roundi(power x Focus x skill x stance x crit x (1 - guard term) x (1 - armour))`,
//! minimum 1, over `power = roundi((weapon + unit) x tier scale)`, minimum 1 (an authored power replaces it unscaled):
//! every hit of a sweep across tier scales, Focus, skill multipliers, stances, crits, guard, cover and armour equals that
//! expression written out here in today's order, and its trace names every step in stage order (`compute_traced`;
//! `compute` keeps none). A modifier in the round stage or an op outside its stages panics, naming the modifier.

use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::pipeline::{self, Modifier, Op, Pipeline, Stage, StepKind};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::{build_unit, Stance, Unit};
use serde_json::json;

fn catalog() -> Catalog {
    Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 11, "INT": 8, "CONS": 12, "PERC": 9, "KNOW": 10}, "growth": {"STR": 1}}],
        "units": [{"id": "probe", "name": "Probe", "class_id": "drill", "level": 3, "side": "player"}]
    }))
}

fn probe() -> Unit {
    let cat = catalog();
    build_unit(&cat, &cat.units["probe"], 0)
}

/// Today's formula before its rounding, written out in today's multiplication order from the hit's own factors.
fn gross(h: &damage::Hit) -> f64 {
    h.attack_power as f64 * h.buff_mult * h.skill_mult * h.stance_factor * h.crit_mult * h.mit_mult * (1.0 - h.armor_ratio)
}

fn formula(h: &damage::Hit) -> i32 {
    roundi(gross(h)).max(1)
}

fn base_formula(u: &Unit, h: &damage::Hit) -> i32 {
    if u.power_override > 0 {
        return u.power_override;
    }
    roundi((h.power.weapon_power + h.power.unit_power) * u.tier_mult).max(1)
}

const STANCES: [Stance; 4] = [Stance::Neutral, Stance::Offensive, Stance::Defensive, Stance::Tactical];

#[test]
fn stages_run_in_their_fixed_order() {
    let m = |id, stage, op| Modifier::new(id, stage, op);
    // listed backwards: the base add still comes first, the mitigation last
    let mods = [m("armour", Stage::Mitigation, Op::Reduce(0.5)), m("double", Stage::Mult, Op::Mul(2.0)), m("flat", Stage::Flat, Op::Add(1.0)), m("start", Stage::Base, Op::Add(10.0))];
    let (out, trace) = pipeline::DAMAGE.run_traced(&mods);
    assert_eq!(out.value, roundi((10.0 + 1.0) * 2.0 * 0.5));
    let order: Vec<Stage> = trace.steps.iter().map(|s| s.stage).collect();
    assert_eq!(order, [Stage::Base, Stage::Base, Stage::Flat, Stage::Mult, Stage::Mitigation, Stage::Round]);
    // the flat stage adds after rounding point one: 2.4 rounds to 2 first, so 2 + 0.4 finishes at 2 (not 3)
    let flat = [m("start", Stage::Base, Op::Add(2.4)), m("flat", Stage::Flat, Op::Add(0.4))];
    assert_eq!(pipeline::DAMAGE.run(&flat).value, 2);
    assert_eq!(pipeline::DAMAGE.run(&flat).base, Some(2));
    // floors: damage at least 1 at both points, stagger at least 0 and no base rounding
    assert_eq!(pipeline::DAMAGE.run(&[m("start", Stage::Base, Op::Add(0.2))]).value, 1);
    assert_eq!(pipeline::STAGGER.run(&[m("start", Stage::Base, Op::Add(0.2))]), pipeline::Outcome { base: None, gross: 0.2, value: 0 });
    let no_base = Pipeline { base_round: None, finish_min: 1 };
    assert_eq!(no_base.run(&[m("a", Stage::Base, Op::Add(1.4)), m("b", Stage::Flat, Op::Add(1.4))]).value, roundi(1.4 + 1.4));
}

#[test]
fn ops_fit_only_their_stages() {
    for stage in Stage::ORDER {
        assert_eq!(Op::Add(1.0).fits(stage), matches!(stage, Stage::Base | Stage::Flat), "add in {}", stage.name());
        assert_eq!(Op::Mul(1.0).fits(stage), matches!(stage, Stage::Base | Stage::Mult | Stage::Mitigation), "mul in {}", stage.name());
        assert_eq!(Op::Reduce(0.0).fits(stage), stage == Stage::Mitigation, "reduce in {}", stage.name());
    }
}

#[test]
#[should_panic(expected = "modifier late: the round stage takes no modifiers")]
fn a_modifier_in_the_round_stage_is_refused() {
    pipeline::DAMAGE.run(&[Modifier::new("start", Stage::Base, Op::Add(3.0)), Modifier::new("late", Stage::Round, Op::Mul(2.0))]);
}

#[test]
#[should_panic(expected = "modifier sneak: Add(1.0) cannot sit in stage mult")]
fn an_op_outside_its_stages_is_refused() {
    pipeline::STAGGER.run(&[Modifier::new("start", Stage::Base, Op::Add(3.0)), Modifier::new("sneak", Stage::Mult, Op::Add(1.0))]);
}

#[test]
fn compute_is_the_formula_in_its_order() {
    let base = probe();
    let mut cases = 0;
    for scale in [0.4, 0.6, 0.7, 0.8, 1.0, 1.15, 1.3] {
        for authored in [0, 17] {
            for boost in [1.0, 1.1, 1.35] {
                for (i, &stance) in STANCES.iter().enumerate() {
                    let target_stance = STANCES[(i + 1) % 4];
                    for guard in [0.0, 0.25, 0.5, 0.85] {
                        for armour in [0, 7, 25, 60] {
                            for crit in [false, true] {
                                for skill in [0.2, 0.5, 0.75, 1.0, 1.25, 1.6] {
                                    let mut a = base.clone();
                                    (a.tier_mult, a.power_override, a.attack_boost) = (scale, authored, boost);
                                    let mut t = base.clone();
                                    (t.guard, t.armor_override, t.stance) = (guard, armour, target_stance);
                                    for (cover, pierce) in [(0.0, false), (0.2, false), (0.3, true)] {
                                        let input = HitInput { skill_mult: skill, force_crit: crit, attacker_stance: Some(stance), cover, ignore_armor: pierce, ..HitInput::default() };
                                        let h = damage::compute_traced(&a, &t, input);
                                        let plain = damage::compute(&a, &t, input);
                                        let case = format!("scale {scale} authored {authored} boost {boost} {stance:?} guard {guard} armour {armour} crit {crit} skill {skill} cover {cover} pierce {pierce}");
                                        assert!(plain.trace.steps.is_empty(), "{case}: compute keeps no trace");
                                        assert_eq!(damage::Hit { trace: h.trace.clone(), ..plain }, h, "{case}: traced or not, the same hit");
                                        assert_eq!(h.attack_power, base_formula(&a, &h), "{case}: power");
                                        assert_eq!(h.attack_power, damage::power_of(&a, false, false), "{case}: power_of");
                                        assert_eq!(h.final_dmg, formula(&h), "{case}: hit");
                                        // bit for bit before the rounding: the same f64 products in the same order
                                        assert_eq!(h.trace.steps.last().unwrap().before.to_bits(), gross(&h).to_bits(), "{case}: gross");
                                        assert_eq!(h.mit_mult, 1.0 - damage::GUARD_CAP.min(guard + cover), "{case}: guard term");
                                        cases += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(cases > 10_000, "{cases} cases");
}

#[test]
fn a_hit_traces_every_step_in_stage_order() {
    let mut a = probe();
    a.tier_mult = 0.8;
    let t = probe();
    let h = damage::compute_traced(&a, &t, HitInput { skill_mult: 1.25, force_crit: true, cover: 0.2, ..HitInput::default() });
    let ids: Vec<&str> = h.trace.steps.iter().map(|s| s.id.as_ref()).collect();
    assert_eq!(ids, ["weapon", "unit", "tier", "round", "focus", "skill", "stance", "crit", "guard_cover", "armour", "round"]);
    let base_round = &h.trace.steps[3];
    assert_eq!((base_round.stage, base_round.kind, base_round.after), (Stage::Base, StepKind::Round { min: 1 }, h.attack_power as f64));
    let last = h.trace.steps.last().unwrap();
    assert_eq!((last.stage, last.kind, last.after), (Stage::Round, StepKind::Round { min: 1 }, h.final_dmg as f64));
    for (id, op) in [("focus", Op::Mul(h.buff_mult)), ("skill", Op::Mul(h.skill_mult)), ("stance", Op::Mul(h.stance_factor)), ("crit", Op::Mul(h.crit_mult)), ("armour", Op::Reduce(h.armor_ratio))] {
        assert_eq!(h.trace.step(id).unwrap().kind, StepKind::Apply(op), "{id}");
    }
    assert_eq!(roundi(h.trace.step("focus").unwrap().after), h.final_attack_power, "the attack power after Focus");
    assert_eq!(h.trace.stage(Stage::Mitigation).count(), 2);
    // each step starts where the one before ended
    for w in h.trace.steps.windows(2) {
        assert_eq!(w[0].after, w[1].before, "{} -> {}", w[0].id, w[1].id);
    }
    let mut authored = a.clone();
    authored.power_override = 23;
    let h = damage::compute_traced(&authored, &t, HitInput::default());
    assert_eq!(h.trace.steps[0].id, "authored");
    assert_eq!((h.trace.steps[1].id.as_ref(), h.attack_power), ("round", 23), "an authored power is unscaled");
}

/// U1.2-S2 (FB-45): an attack power below `MIN_POWER` counts as `MIN_POWER` at rounding point one, before the
/// multipliers, so a later multiplier scales the floored value.
#[test]
fn an_attack_power_below_the_floor_counts_as_min_power() {
    assert_eq!(pipeline::MIN_POWER, 1);
    for raw in [-5.0, 0.0, 0.4] {
        let mods = [Modifier::new("power", Stage::Base, Op::Add(raw)), Modifier::new("amp", Stage::Mult, Op::Mul(3.0))];
        let out = pipeline::DAMAGE.run(&mods);
        assert_eq!(out.base, Some(pipeline::MIN_POWER), "raw {raw}");
        assert_eq!(out.value, 3 * pipeline::MIN_POWER, "raw {raw}: the multiplier scales the floor");
    }
}
