//! BASERULES1-M (docs/specs/BASERULES1_LIBRARY.md 7): the rule register's metadata. Every rule names a system, a known
//! bundle, valid requirement and conflict tokens and test files that exist; the bundles, field rules and tested sets
//! name only what the code owns; a complete set raises nothing, a rule without what it requires is an error, a conflict,
//! a split bundle, a set field nothing reads and an untested set are warnings, and each conflict token raises its
//! warning exactly when it holds (the mutation check). Additive: the built-in classic values and an unlisted rule raise
//! nothing.

use rpgkit_rules::ruleset::{
    self, by_rules_values, bundle, field_rule_of, rule, AttackLock, FlankCost, MeleeDiagonal, Numbers, Ruleset, BUNDLES, BY_RULES_FIELDS, DECK, FIELD_RULES, FLANK,
    PLAN_BADGES, POISE, REGISTER, ROUND_INTENT, TESTED_SETS,
};
use serde_json::{json, Value};
use std::path::PathBuf;

/// The values a `<field>=<value>` token may name (None: the field takes any value).
fn values_of(key: &str) -> Option<Vec<&'static str>> {
    if BY_RULES_FIELDS.contains(&key) {
        return Some(by_rules_values(key).to_vec());
    }
    match key {
        "melee_diagonal" => Some(MeleeDiagonal::IDS.to_vec()),
        "flank_protected_cost" => Some(FlankCost::IDS.to_vec()),
        "attack_lock" => Some(AttackLock::IDS.to_vec()),
        _ => None,
    }
}

/// A token names a rule, a field, or a field and one of its values (`!` negates).
fn valid_token(t: &str) -> bool {
    let t = t.strip_prefix('!').unwrap_or(t);
    match t.split_once('=') {
        Some((key, v)) => Numbers::default().field(key).is_some() && values_of(key).is_none_or(|vs| vs.contains(&v)),
        None => rule(t).is_some() || Numbers::default().field(t).is_some(),
    }
}

fn row(v: Value) -> Ruleset {
    Ruleset::from_row(v.as_object().expect("an object"))
}

/// A row listing `rules` with field `key` set to `v`.
fn row_with(rules: &[&str], key: &str, v: &str) -> Ruleset {
    let mut o = json!({"rules": rules});
    o[key] = json!(v);
    row(o)
}

/// Every rule, the built-in timeline way: the by-rules choices, a poise stat, basics and slots.
fn complete() -> Ruleset {
    let all: Vec<&str> = REGISTER.iter().map(|r| r.id).collect();
    row(json!({"id": "full", "rules": all, "poise_stat": "max_poise", "basics": ["strike"], "deck_slots": {"active": 3, "passive": 2, "keystone": 1}, "break_chance_scale": 0.5}))
}

#[test]
fn each_rule_has_its_metadata() {
    let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rpgkit_battle/tests");
    for r in REGISTER.iter() {
        assert!(!r.system.is_empty() && !r.does.is_empty(), "{}: system and what it does", r.id);
        assert!(r.bundle.is_empty() || bundle(r.bundle).is_some(), "{}: bundle '{}'", r.id, r.bundle);
        assert!(r.bundle.is_empty() || bundle(r.bundle).is_some_and(|b| b.holds.contains(&r.id) || b.anchor.contains(&r.id)), "{}: its bundle lists it", r.id);
        for t in r.requires.iter() {
            assert!(rule(t).is_some(), "{}: requires a rule, not '{t}'", r.id);
        }
        for t in r.conflicts.iter() {
            assert!(valid_token(t), "{}: conflict '{t}'", r.id);
        }
        assert!(!r.tested.is_empty(), "{}: tested somewhere", r.id);
        for f in r.tested.iter().filter(|f| **f != "golden") {
            assert!(tests.join(f).is_file(), "{}: test file {f}", r.id);
        }
    }
    let ids: Vec<&str> = BUNDLES.iter().map(|b| b.id).collect();
    for b in BUNDLES.iter() {
        assert_eq!(ids.iter().filter(|i| **i == b.id).count(), 1, "bundle {} once", b.id);
        for t in b.anchor.iter().chain(b.holds.iter()) {
            assert!(valid_token(t), "bundle {}: '{t}'", b.id);
        }
    }
    for f in FIELD_RULES.iter() {
        assert!(Numbers::default().field(f.key).is_some(), "field rule {}: a ruleset field", f.key);
        assert_eq!(field_rule_of(f.key), Some(f));
        for t in f.needs.iter().chain(f.untested_with.iter()) {
            assert!(valid_token(t), "field rule {}: '{t}'", f.key);
        }
    }
    for set in TESTED_SETS.iter() {
        assert!(set.iter().all(|r| rule(r).is_some()), "tested set {set:?}");
        assert!(Ruleset { rules: set.to_vec(), ..Ruleset::classic() }.notes().errors.is_empty(), "a tested set is not refused: {set:?}");
    }
    assert!(ruleset::tested_set(&REGISTER.iter().map(|r| r.id).collect::<Vec<_>>()), "every rule together is tested (the timeline golden)");
}

#[test]
fn a_complete_set_and_the_classic_values_raise_nothing() {
    assert_eq!(Ruleset::classic().notes(), Default::default(), "the built-in classic values");
    assert_eq!(complete().notes(), Default::default(), "every rule, the by-rules choices");
    assert_eq!(row(json!({"rules": []})).notes(), Default::default(), "a row without rules");
    assert_eq!(row(json!({"rules": [ROUND_INTENT, FLANK]})).notes(), Default::default(), "flank with what it requires");
}

#[test]
fn a_rule_without_what_it_requires_is_an_error() {
    let n = row(json!({"rules": [FLANK]})).notes();
    assert_eq!(n.errors, ["flank needs round_intent in Rules: without it the rule does nothing."]);
    let mut r = Ruleset::classic();
    r.adjust("+flank").expect("adjust never refuses an extension");
    assert_eq!(r.notes().errors, n.errors, "an adjusted set gets the same note");
    r.adjust("-flank").unwrap();
    assert!(r.notes().errors.is_empty());
}

#[test]
fn a_split_bundle_a_stray_field_and_an_untested_set_warn() {
    let n = row(json!({"rules": [ROUND_INTENT], "chase": "classic"})).notes();
    assert_eq!(n.warnings, ["Bundle intent_core ships together: it needs chase=every_skill."]);
    let n = row(json!({"rules": [], "rest_ap": "round_end"})).notes();
    assert_eq!(n.warnings, ["Bundle legacy_core ships together: it needs rest_ap=on_use."]);
    let n = row(json!({"rules": [], "break_chance_scale": 0.5, "rest_ap_cap": 3})).notes();
    assert_eq!(n.warnings, ["break_chance_scale is set but read only with poise.", "rest_ap_cap is set but read only with rest_ap=round_end."]);
    let n = row(json!({"rules": [POISE], "break_chance_scale": 0.5})).notes();
    assert!(n.warnings.is_empty(), "{n:?}");
    let n = row(json!({"rules": [], "attack_lock": "melee"})).notes();
    assert_eq!(n.warnings, ["attack_lock melee is untested with chase=classic."]);
    let n = row(json!({"rules": [PLAN_BADGES]})).notes();
    assert_eq!(n.warnings, ["Rules untested together: plan_badges (no kit test fights this set; aegis rules lists the tested sets)."]);
    let n = row(json!({"rules": [DECK], "basics": ["strike"], "deck_slots": {"active": 3}})).notes();
    assert!(n.warnings.is_empty(), "a complete build bundle: {n:?}");
    let n = row(json!({"rules": [DECK], "deck_slots": {"active": 3}})).notes();
    assert_eq!(n.warnings, ["Bundle build ships together: it needs basics."]);
    // an adjusted set leaves the tested sets, and is told so, but plays
    let mut r = complete();
    r.adjust("-pierce").unwrap();
    assert!(r.notes().warnings.iter().any(|w| w.starts_with("Rules untested together:")), "{:?}", r.notes());
}

#[test]
fn a_conflict_warns() {
    let n = row(json!({"rules": [ROUND_INTENT], "stances": "on"})).notes();
    assert_eq!(n.warnings, ["round_intent conflicts with stances=on: it runs, but the two cut against each other."]);
    assert!(n.errors.is_empty(), "a conflict never refuses");
}

/// The mutation check: each conflict token of each rule raises its one warning exactly when the row sets it, and
/// nothing when the row sets another value or does not list the rule.
#[test]
fn each_conflict_warns_exactly_when_it_holds() {
    let mut checked = 0;
    for r in REGISTER.iter() {
        let base_rules: Vec<&str> = if r.requires.is_empty() { vec![r.id] } else { r.requires.iter().copied().chain([r.id]).collect() };
        for c in r.conflicts.iter() {
            let (key, v) = c.split_once('=').expect("a field=value conflict");
            let want = format!("{} conflicts with {c}: it runs, but the two cut against each other.", r.id);
            let on = row_with(&base_rules, key, v);
            assert_eq!(on.notes().warnings.iter().filter(|w| **w == want).count(), 1, "{c}: {:?}", on.notes());
            for other in values_of(key).expect("a value field").into_iter().filter(|o| *o != v) {
                let off = row_with(&base_rules, key, other);
                assert!(!off.notes().warnings.contains(&want), "{key}={other}: {:?}", off.notes());
            }
            let without = row_with(&[], key, v);
            assert!(!without.notes().warnings.contains(&want), "{c} without {}", r.id);
            checked += 1;
        }
    }
    assert!(checked > 0, "the register has a conflict to check");
}
