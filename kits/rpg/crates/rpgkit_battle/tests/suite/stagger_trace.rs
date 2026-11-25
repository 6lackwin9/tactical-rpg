//! U4.2-S4 (FB-30) on a scratch catalog built here (no shipped data): `poise::stagger_traced` gives the stagger the
//! untraced pipeline gives (`STAGGER.run(&stagger_modifiers(..))`) for a target with 0, 1 and 2 `stagger_taken` rows
//! matching the action, and its `passive` step names one part per matching row, in apply order (a row that does not
//! match the action's length names none). Built in debug, so the evaluator's `debug_assert` runs.

use rpgkit_battle::poise::{self, Factors};
use rpgkit_data::Catalog;
use rpgkit_rules::pipeline::{Op, StepKind, STAGGER};
use rpgkit_rules::{build_unit, Unit};
use serde_json::json;

const BLOCKS: i32 = 2;
const STAGGER_IN: i32 = 13;

fn target() -> (Catalog, Unit) {
    let cat = Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 10}, "growth": {}}],
        "units": [{"id": "t", "name": "T", "class_id": "drill", "level": 1, "side": "enemy"}]
    }));
    let u = build_unit(&cat, &cat.units["t"], 0);
    (cat, u)
}

/// `poise::factors`' reading of the target's rows, with the other factors off the default so every step shows.
fn factors(u: &Unit) -> Factors {
    let passive = u.stagger_taken.iter().filter(|e| e.matches(BLOCKS)).map(|e| e.mult).product();
    Factors { windup: 2.0, guard: 0.5, repeat: 1.0, flank: 1.5, passive, hook: 1.0 }
}

#[test]
fn stagger_traced_equals_the_untraced_run_and_names_each_matching_row() {
    let (cat, base) = target();
    // (rows applied, the sources whose rows match a 2-block action)
    let cases: [(Vec<(&str, f64, i32)>, Vec<&str>); 4] = [
        (vec![], vec![]),
        (vec![("passive:a", 0.5, 0)], vec!["passive:a"]),
        (vec![("passive:a", 0.5, 0), ("passive:b", 0.8, 3)], vec!["passive:a", "passive:b"]),
        // a 1-block row does not match a 2-block action: no part, no factor
        (vec![("passive:a", 0.5, 0), ("passive:c", 0.25, 1), ("passive:b", 0.8, 3)], vec!["passive:a", "passive:b"]),
    ];
    for (rows, want) in cases {
        let mut u = base.clone();
        for (src, mult, max_blocks) in &rows {
            u.apply_effects(&cat, &[json!({"type": "stagger_taken", "mult": mult, "max_blocks": max_blocks})], src, false);
        }
        let f = factors(&u);
        let plain = STAGGER.run(&poise::stagger_modifiers(STAGGER_IN, &f)).value;
        let (traced, trace) = poise::stagger_traced(STAGGER_IN, &f, &u, BLOCKS);
        assert_eq!(traced, plain, "{rows:?}: the traced stagger is the untraced one");
        assert_eq!(traced, poise::stagger_taken(STAGGER_IN, &f), "{rows:?}");
        let step = trace.step("passive").expect("the passive step");
        assert_eq!(step.kind, StepKind::Apply(Op::Mul(f.passive)), "{rows:?}");
        let sources: Vec<&str> = step.parts.iter().map(|p| p.source.as_ref()).collect();
        assert_eq!(sources, want, "{rows:?}: one part per matching row");
        assert!(trace.steps.iter().filter(|s| s.id != "passive").all(|s| s.parts.is_empty()), "rule terms carry no parts");
    }
}
