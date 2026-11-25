//! INTENT2-K1 / A4.2-K1b (FB-106, BASERULES1 library 3): the rule `field_intent` gates the IR21 derivation of a
//! skill's allowed intents (`intent::allowed`). Without it a row plays the legacy single intent even under
//! `round_intent`; with both, the derived list. The register's convention for a rule without what it requires holds:
//! `field_intent` alone is an error note (the content check refuses the row), and as an option of its ruleset it does
//! not switch one by one. Classic has it off.

use rpgkit_rules::intent::{self, IntentInput};
use rpgkit_rules::ruleset::{self, rule, RuleKind, Ruleset, FIELD_INTENT, ROUND_INTENT};
use serde_json::{json, Value};

fn row(v: Value) -> Ruleset {
    Ruleset::from_row(v.as_object().expect("an object"))
}

/// Test-only skill rows: a shaped attack IR21 gives two intents, a plain attack, a focus and a gap-closer.
fn skills() -> Vec<IntentInput> {
    [json!({"kind": "ATTACK", "area": "LINE"}), json!({"kind": "ATTACK"}), json!({"kind": "FOCUS"}), json!({"kind": "ATTACK", "dash_tiles": 2})]
        .iter()
        .map(|v| IntentInput::of_row(v.as_object().unwrap()))
        .collect()
}

/// The legacy single intent (RULESET1_INTENT IN4): the authored one, else the kind's.
fn legacy(s: &IntentInput, authored: &'static str) -> Vec<&'static str> {
    vec![if authored.is_empty() { ruleset::kind_intent(&s.kind) } else { authored }]
}

#[test]
fn without_field_intent_the_legacy_single_intent_holds_even_under_round_intent() {
    let classic = Ruleset::classic();
    assert!(!classic.has(FIELD_INTENT), "classic has field_intent off");
    let round = row(json!({"rules": [ROUND_INTENT]}));
    assert!(round.has(ROUND_INTENT) && !round.has(FIELD_INTENT));
    for s in skills() {
        for authored in ["", "maneuver"] {
            let single = intent::allowed(&round, &s, authored);
            assert_eq!(single, legacy(&s, authored), "{} {authored:?}: legacy under round_intent alone", s.kind);
            assert_eq!(single, intent::allowed(&classic, &s, authored), "{}: as classic reads it", s.kind);
        }
    }
}

#[test]
fn with_round_intent_and_field_intent_the_ir21_derivation_applies() {
    let on = row(json!({"rules": [ROUND_INTENT, FIELD_INTENT]}));
    assert!(on.has(ROUND_INTENT) && on.has(FIELD_INTENT));
    for s in skills() {
        for authored in ["", "maneuver", "guard"] {
            assert_eq!(intent::allowed(&on, &s, authored), intent::derive(&s), "{} {authored:?}: derived, the authored intent unread", s.kind);
        }
    }
    let line = &skills()[0];
    assert_eq!(intent::allowed(&on, line, "").len(), 2, "a shaped LINE attack is dual under the rule");
    assert_eq!(intent::allowed(&row(json!({"rules": [ROUND_INTENT]})), line, "").len(), 1, "single without it");
}

#[test]
fn field_intent_requires_round_intent_by_the_register() {
    let r = rule(FIELD_INTENT).expect("field_intent is registered");
    assert_eq!(r.requires, [ROUND_INTENT]);
    assert_eq!(r.kind, RuleKind::Option, "an option of its ruleset: never in classic");
    assert!(ruleset::REGISTER.iter().position(|x| x.id == ROUND_INTENT) < ruleset::REGISTER.iter().position(|x| x.id == FIELD_INTENT));
    let alone = row(json!({"rules": [FIELD_INTENT]})).notes();
    assert_eq!(alone.errors, [format!("{FIELD_INTENT} needs {ROUND_INTENT} in Rules: without it the rule does nothing.")]);
    let both = row(json!({"rules": [ROUND_INTENT, FIELD_INTENT]}));
    assert_eq!(both.notes(), Default::default(), "with what it requires: a clean, tested set");
    assert!(ruleset::tested_set(&both.rules));
    let mut classic = Ruleset::classic();
    assert!(classic.adjust(&format!("+{FIELD_INTENT}")).is_err(), "an option does not switch one by one");
    assert!(!classic.has(FIELD_INTENT));
}

/// A4.2-K1b (INTENT2 IR17; A3.1 section 3 tunables, FB-102): `splash_mult` (default `SPLASH_MULT`, 0.25..1.00) and
/// `assault_shape_blocks` (default `ASSAULT_SHAPE_BLOCKS`, 0..2) are row numbers read only under `field_intent`
/// (row-only: no schema entry yet), promoted from their code defaults.
#[test]
fn the_shaped_assault_numbers_are_row_numbers_read_under_field_intent() {
    assert_eq!(ruleset::SPLASH_MULT, 0.5, "IR17 / FB-102: half splash first");
    assert_eq!(ruleset::ASSAULT_SHAPE_BLOCKS, 1, "IR17: one block more");
    let classic = Ruleset::classic();
    assert_eq!((classic.numbers.splash_mult, classic.numbers.assault_shape_blocks), (ruleset::SPLASH_MULT, ruleset::ASSAULT_SHAPE_BLOCKS));
    let set = row(json!({"rules": [ROUND_INTENT, FIELD_INTENT], "splash_mult": 0.75, "assault_shape_blocks": 2}));
    assert_eq!((set.numbers.splash_mult, set.numbers.assault_shape_blocks), (0.75, 2));
    assert_eq!(set.numbers.field("splash_mult").as_deref(), Some("0.75"));
    assert_eq!(set.numbers.field("assault_shape_blocks").as_deref(), Some("2"));
    assert!(set.notes().errors.is_empty() && set.notes().warnings.iter().all(|w| !w.contains("splash_mult") && !w.contains("assault_shape_blocks")), "{:?}", set.notes());
    for (key, name, range) in [("splash_mult", "SPLASH_MULT", (0.25, 1.0)), ("assault_shape_blocks", "ASSAULT_SHAPE_BLOCKS", (0.0, 2.0))] {
        let f = ruleset::field_rule_of(key).expect("registered");
        assert_eq!((f.needs, f.range), (&[FIELD_INTENT][..], Some(range)), "{key}");
        assert!(ruleset::PROMOTED.contains(&(name, key)));
    }
    // set without the rule: a warning (nothing reads it); outside the range: an error
    let unread = row(json!({"rules": [ROUND_INTENT], "splash_mult": 0.75}));
    assert!(unread.notes().warnings.contains(&format!("splash_mult is set but read only with {FIELD_INTENT}.")), "{:?}", unread.notes());
    assert_eq!(row(json!({"rules": [ROUND_INTENT, FIELD_INTENT], "assault_shape_blocks": 3})).notes().errors, ["assault_shape_blocks 3 is outside 0..2."]);
    assert_eq!(row(json!({"rules": [ROUND_INTENT, FIELD_INTENT], "splash_mult": 0.1})).notes().errors, ["splash_mult 0.1 is outside 0.25..1."]);
}
