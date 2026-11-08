//! A2.2-S1/S2 (FB-43, FB-56): damage taken, heals and folded skill factors on `rpgkit_rules::pipeline`, on a scratch
//! catalog (no shipped data). `take_damage` is an exact port of Combatant.take_damage
//! (`max(1, roundi(amount x (1 - guard)))` only when the guard applies, the amount as it is otherwise), `Unit::heal`
//! of `min(max_hp, hp + amount)` for a living unit, and the heal amount of `roundi(base x potency)` floor 0; each keeps
//! a trace of its steps. A hit traced with folded factors (`compute_traced_folded`) names them as parts of its `skill`
//! step while every number stays the untraced hit's.

use rpgkit_data::Catalog;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::pipeline::{Op, Part, Stage, StepKind};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::{build_unit, Stance, Unit};
use serde_json::json;

fn catalog() -> Catalog {
    Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 11, "INT": 8, "CONS": 12, "PERC": 9, "KNOW": 10}, "growth": {"STR": 1}}],
        "units": [{"id": "probe", "name": "Probe", "class_id": "drill", "level": 3, "side": "player"}],
        "skills": [{"id": "t_mend", "name": "Mend", "kind": "SUPPORT", "level": 1, "blocks": 2, "heal_base": 8, "heal_stat": "KNOW", "heal_mult": 0.3, "target_mode": "ALLY_OR_SELF"}]
    }))
}

fn probe() -> Unit {
    let cat = catalog();
    build_unit(&cat, &cat.units["probe"], 0)
}

/// Combatant.take_damage before the port, written out.
fn old_take(amount: i32, guard: f64, mitigated: bool) -> i32 {
    if !mitigated && guard > 0.0 {
        roundi(amount as f64 * (1.0 - guard)).max(1)
    } else {
        amount
    }
}

const AMOUNTS: [i32; 14] = [-3, 0, 1, 2, 3, 4, 5, 7, 10, 13, 25, 99, 250, 1000];
const GUARDS: [f64; 10] = [0.0, 0.05, 0.1, 0.25, 1.0 / 3.0, 0.5, 0.6, 0.75, 0.9, 1.0];

#[test]
fn take_damage_matches_the_old_formula() {
    for amount in AMOUNTS {
        for guard in GUARDS {
            for mitigated in [false, true] {
                let want = old_take(amount, guard, mitigated);
                assert_eq!(damage::take_damage(amount, guard, mitigated), want, "amount {amount} guard {guard} mitigated {mitigated}");
                let (traced, _) = damage::take_damage_traced(amount, guard, mitigated);
                assert_eq!(traced, want);
                // the unit applies it: HP drops by the damage, never below 0
                let mut u = probe();
                u.guard = guard;
                let hp = u.hp;
                assert_eq!(u.take_damage(amount, mitigated), want);
                assert_eq!(u.hp, (hp - want).max(0));
            }
        }
    }
}

#[test]
fn take_damage_trace_names_its_steps() {
    // the guard applies: the amount, the guard ratio, one rounding with floor 1
    let (dmg, trace) = damage::take_damage_traced(10, 0.25, false);
    let ids: Vec<&str> = trace.steps.iter().map(|s| s.id.as_ref()).collect();
    assert_eq!(ids, ["amount", "guard", "round"]);
    assert_eq!(trace.step("guard").unwrap().kind, StepKind::Apply(Op::Reduce(0.25)));
    assert_eq!(trace.step("guard").unwrap().stage, Stage::Mitigation);
    let last = trace.steps.last().unwrap();
    assert_eq!((last.stage, last.kind, last.after), (Stage::Round, StepKind::Round { min: 1 }, dmg as f64));
    // the floor holds only on the guard path: a tiny hit against a full guard is 1
    assert_eq!(damage::take_damage(1, 1.0, false), 1);
    // no guard (or already mitigated): the amount as it is, no guard step and no floor
    for (guard, mitigated) in [(0.0, false), (0.5, true)] {
        let (dmg, trace) = damage::take_damage_traced(0, guard, mitigated);
        assert_eq!(dmg, 0);
        let ids: Vec<&str> = trace.steps.iter().map(|s| s.id.as_ref()).collect();
        assert_eq!(ids, ["amount", "round"]);
    }
}

#[test]
fn unit_heal_matches_the_old_cap() {
    for start in [1, 5, 20] {
        for amount in [-2, 0, 1, 3, 10, 500] {
            let mut u = probe();
            u.hp = start.min(u.max_hp);
            let want = u.max_hp.min(u.hp + amount);
            let (gain, trace) = damage::heal_gain_traced(u.hp, u.max_hp, amount);
            assert_eq!(u.hp + gain, want);
            assert_eq!(trace.step("heal").unwrap().kind, StepKind::Apply(Op::Add(amount as f64)));
            u.heal(amount);
            assert_eq!(u.hp, want, "start {start} amount {amount}");
        }
    }
    // a downed unit is not healed
    let mut u = probe();
    u.hp = 0;
    u.heal(10);
    assert_eq!(u.hp, 0);
}

#[test]
fn heal_amount_trace_is_base_then_potency() {
    let cat = catalog();
    let u = probe();
    let s = cat.skills.iter().find(|s| s.id == "t_mend").expect("scratch skill");
    let (value, trace) = damage::heal_traced(&u, s);
    assert_eq!(value, damage::heal(&u, s));
    let ids: Vec<&str> = trace.steps.iter().map(|s| s.id.as_ref()).collect();
    assert_eq!(ids, ["heal", "potency", "round"]);
    let base = trace.step("heal").unwrap().after;
    // heal_base + roundi(stat x heal_mult): an integer at least heal_base
    assert_eq!(base, base.round());
    assert!(base >= s.heal_base as f64);
    let StepKind::Apply(Op::Mul(potency)) = trace.step("potency").unwrap().kind else { panic!("potency is a multiplier") };
    assert_eq!(value, roundi(base * potency).max(0));
}

#[test]
fn folded_factors_are_trace_only() {
    let a = probe();
    let mut t = probe();
    t.guard = 0.25;
    let cat = catalog();
    let s = cat.skills.iter().find(|s| s.id == "t_mend").expect("scratch skill");
    // the attack's order (command.rs): skill x exploit x flank; the counter's: counter_mult x weaken x empower
    let cases: [(f64, Vec<(&'static str, f64)>); 3] = [
        (1.2 * 1.0 * 1.25, vec![(damage::FOLD_SKILL, 1.2), (damage::FOLD_EXPLOIT, 1.0), (damage::FOLD_FLANK, 1.25)]),
        (0.5 * 0.8 * 1.0, vec![(damage::FOLD_COUNTER, 0.5), (damage::FOLD_WEAKEN, 0.8), (damage::FOLD_EMPOWER, 1.0)]),
        (1.0 * 0.5, vec![(damage::FOLD_SKILL, 1.0), (damage::FOLD_SPLASH, 0.5)]),
    ];
    for stance in [Stance::Neutral, Stance::Offensive] {
        for crit in [false, true] {
            for (mult, folded) in &cases {
                let h = HitInput { skill_mult: *mult, force_crit: crit, attacker_stance: Some(stance), ..HitInput::default() };
                let plain = damage::compute(&a, &t, h);
                let traced = damage::compute_traced(&a, &t, h);
                let hit = damage::compute_traced_folded(&a, &t, h, None, folded);
                assert_eq!(hit.final_dmg, plain.final_dmg);
                // the same steps and numbers as the plain traced hit; only the skill step's parts differ
                assert_eq!(hit.trace.steps.len(), traced.trace.steps.len());
                for (x, y) in hit.trace.steps.iter().zip(&traced.trace.steps) {
                    assert_eq!((x.stage, &x.id, x.kind, x.before, x.after), (y.stage, &y.id, y.kind, y.before, y.after));
                }
                let step = hit.trace.step("skill").unwrap();
                assert_eq!(step.kind, StepKind::Apply(Op::Mul(*mult)));
                let want: Vec<Part> = folded.iter().map(|&(id, value)| Part { source: id.into(), value }).collect();
                assert_eq!(step.parts, want);
                // with a skill, its grant sources come first, then the folded factors
                let with_skill = damage::compute_traced_folded(&a, &t, h, Some(s), folded);
                assert_eq!(with_skill.final_dmg, plain.final_dmg);
                let mut want_all = a.potency_parts(s);
                want_all.extend(want);
                assert_eq!(with_skill.trace.step("skill").unwrap().parts, want_all);
            }
        }
    }
}
