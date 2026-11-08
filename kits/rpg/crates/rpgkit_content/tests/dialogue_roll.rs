//! T1.2-S2 / T1.2-S3 (FB-54b,c): the content checks of the dialogue `roll` effect and the standing-rating stat names.
//! A choice's `roll` takes `stat` (a primary stat or a standing rating), `dc` (1..999), `key` (a flag key that is not
//! reserved and not a reputation key) and an optional `who` (a character of the world); it is a choice effect only
//! (a node, a campaign row or a sequence step refuses it). A `stat` condition takes the rating names too, its bar up to
//! 999 for a rating (30 for a primary stat). The world manifest's `dialogue_roll_die` is -1 or 6..100.

use rpgkit_content::check::dialogue::{self, EffectList};
use rpgkit_content::check::{world, View};
use rpgkit_data::{DIALOGUE_ROLL_DIE_MAX, DIALOGUE_ROLL_DIE_MIN, STANDING_RATINGS};
use serde_json::{json, Value};

const LABEL: &str = "Choice";

fn view_with(manifest: Value) -> View {
    let worlds = json!({"w": {"manifest": manifest, "own": {"characters": [{"id": "hero", "name": "Hero", "role": "party", "_world": "w", "_qid": "w:hero"}]}}});
    let catalog = json!({"derived_stats": [{"id": "grit", "name": "Grit", "formula": "CONS * 2 + 1"}]});
    View::new(&catalog, &worlds, &json!({}), Default::default())
}

fn view() -> View {
    view_with(json!({"id": "w", "name": "W"}))
}

fn roll(extra: Value) -> Value {
    let mut fx = json!({"effect": "roll", "stat": "STR", "dc": 15, "key": "beam_lifted"});
    for (k, v) in extra.as_object().unwrap() {
        if v.is_null() {
            fx.as_object_mut().unwrap().shift_remove(k);
        } else {
            fx[k] = v.clone();
        }
    }
    fx
}

fn errors(list: EffectList, fx: Value) -> Vec<String> {
    dialogue::list_errors(&view(), "w", &json!([fx]), LABEL, list)
}

fn choice(fx: Value) -> Vec<String> {
    errors(EffectList::Choice, fx)
}

#[test]
fn a_valid_roll_passes_on_a_choice() {
    assert!(choice(roll(json!({}))).is_empty());
    assert!(choice(roll(json!({"who": "hero"}))).is_empty(), "who: a character of the world");
    for name in STANDING_RATINGS.iter().copied().chain(["grit"]) {
        assert!(choice(roll(json!({"stat": name, "dc": 200}))).is_empty(), "{name}: a standing rating");
    }
}

#[test]
fn a_bad_roll_names_each_problem() {
    let el = format!("{LABEL} effect 1 (roll)");
    let bad_stat = choice(roll(json!({"stat": "LUCK"})));
    assert_eq!(bad_stat.len(), 1);
    assert!(bad_stat[0].starts_with(&format!("{el}: stat 'LUCK' is not one of STR")), "{bad_stat:?}");
    for dc in [json!(0), json!(1000), json!(2.5), Value::Null] {
        assert_eq!(choice(roll(json!({"dc": dc}))), [format!("{el}: dc must be a whole number 1..999.")], "dc {dc}");
    }
    assert_eq!(choice(roll(json!({"key": null}))), [format!("{el}: flag key '' must be non-empty [a-z0-9_.].")]);
    assert_eq!(choice(roll(json!({"key": "rep.w.guild"}))), [format!("{el}: Reputation 'rep.w.guild' must be a whole number from -100 to 100.")], "a reputation key holds no true/false");
    assert!(!choice(roll(json!({"key": "meter.dread"}))).is_empty(), "a reserved key");
    assert!(choice(roll(json!({"who": "nobody"}))).iter().any(|e| e.starts_with(&format!("{el} who"))), "an unknown who");
    assert_eq!(choice(roll(json!({"who": 3}))), [format!("{el}: who must be a character ref or \"\".")]);
    assert_eq!(choice(roll(json!({"luck": 1}))), [format!("{el}: unknown key 'luck'.")]);
}

#[test]
fn a_roll_is_a_choice_effect_only() {
    let el = format!("{LABEL} effect 1 (roll)");
    assert_eq!(errors(EffectList::Node, roll(json!({}))), [format!("{el}: Node effects may only be set_flag, add_flag or give.")]);
    for list in [EffectList::Campaign, EffectList::Sequence] {
        assert_eq!(errors(list, roll(json!({}))), [format!("{el}: roll is a choice effect (it rolls when the player picks the choice).")], "{list:?}");
    }
}

#[test]
fn a_stat_condition_reads_the_standing_ratings() {
    let conds = |c: Value| dialogue::conditions_errors(&view(), "w", &json!([c]), "Requires");
    for name in STANDING_RATINGS.iter().copied().chain(["grit"]) {
        assert!(conds(json!({"kind": "stat", "stat": name, "threshold": 999})).is_empty(), "{name}: a rating bar up to 999");
        assert_eq!(conds(json!({"kind": "stat", "stat": name, "min": 1000})), ["Requires 1 (stat): min must be a whole number 1..999."]);
    }
    assert_eq!(conds(json!({"kind": "stat", "stat": "STR", "min": 31})), ["Requires 1 (stat): min must be a whole number 1..30."], "a primary stat keeps 30");
    let bad = conds(json!({"kind": "stat", "stat": "luck", "min": 5}));
    assert_eq!(bad.len(), 1);
    assert!(bad[0].contains("stat 'luck' is not one of STR"), "{bad:?}");
}

#[test]
fn the_world_dialogue_roll_die_is_inherited_or_in_range() {
    let die_errors = |die: Value| world::manifest_errors(&view_with(json!({"id": "w", "name": "W", "dialogue_roll_die": die})), "w").into_iter().filter(|e| e.starts_with("Dialogue roll die")).collect::<Vec<_>>();
    let want = [format!("Dialogue roll die must be -1 (inherited) or {DIALOGUE_ROLL_DIE_MIN}..{DIALOGUE_ROLL_DIE_MAX}.")];
    for ok in [-1, DIALOGUE_ROLL_DIE_MIN, DIALOGUE_ROLL_DIE_MAX] {
        assert!(die_errors(json!(ok)).is_empty(), "{ok}");
    }
    for bad in [json!(0), json!(DIALOGUE_ROLL_DIE_MIN - 1), json!(DIALOGUE_ROLL_DIE_MAX + 1), json!("d20")] {
        assert_eq!(die_errors(bad.clone()), want, "{bad}");
    }
    assert!(world::manifest_errors(&view(), "w").iter().all(|e| !e.starts_with("Dialogue roll die")), "unset: inherited");
}
