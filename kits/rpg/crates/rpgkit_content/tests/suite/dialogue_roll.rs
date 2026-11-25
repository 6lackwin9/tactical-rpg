//! T1.2-S2 / T1.2-S3 (FB-54b,c): the content checks of the dialogue `roll` effect and the standing-rating stat names.
//! A choice's `roll` takes `stat` (a primary stat or a standing rating), `dc` (1..999), `key` (a flag key that is not
//! reserved and not a reputation key) and an optional `who` (a character of the world); it is a choice effect only
//! (a node, a campaign row or a sequence step refuses it). A `stat` condition takes the rating names too, its bar up to
//! 999 for a rating (30 for a primary stat). The world manifest's `dialogue_roll_die` is -1 or 6..100.
//! R4-2 (FB-143) AUDIT-03: a roll choice on a dialogue cycle warns unless an `unless` that its choice's or node's
//! `set_flag` sets hides it once picked; an `unless` on the roll's own key is no guard (AUDIT-06).

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

// ---- R4-2 (FB-143) AUDIT-03/06: a roll choice on a dialogue cycle is guarded by an unless its own set_flag sets -----

/// The tree warnings of world `w` holding the one dialogue `talk` (start node `ask`, its choice 1 = `choice`).
fn loop_warnings(choice: Value, services: bool) -> Vec<String> {
    let talk = json!({"id": "talk", "_world": "w", "_qid": "w:talk", "start": "ask", "services": services, "nodes": [
        {"id": "ask", "text": "The beam?", "choices": [choice, {"text": "Leave.", "to": "bye"}]},
        {"id": "bye", "text": "Bye.", "end": true}
    ]});
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {"dialogues": [talk]}}});
    dialogue::dialogue_warnings(&View::new(&json!({}), &worlds, &json!({}), Default::default()), "w")
}

#[test]
fn an_unguarded_roll_choice_on_a_dialogue_cycle_warns() {
    let key = roll(json!({}))["key"].as_str().unwrap_or_default().to_string();
    let want = [format!("w:talk: Node 'ask' choice 1 rolls '{key}' and sits on a dialogue cycle (it can be picked again until the roll succeeds); guard it with a set_flag the choice sets and an unless on that flag.")];
    // the choice leads back to its own node, or the services return to the start: a cycle either way
    assert_eq!(loop_warnings(json!({"text": "Lift it.", "to": "ask", "effects": [roll(json!({}))]}), false), want, "a choice back to its node");
    assert_eq!(loop_warnings(json!({"text": "Lift it.", "to": "bye", "effects": [roll(json!({}))]}), true), want, "the services edge back to the start");
    // an unless on another key does not guard it
    let other = json!({"text": "Lift it.", "to": "ask", "unless": [{"key": "other_flag"}], "effects": [roll(json!({}))]});
    assert_eq!(loop_warnings(other, false), want, "an unless on another key");
    // AUDIT-06: an unless on the roll's own key is no guard (a failed roll sets it false, so the choice stays open)
    let own = json!({"text": "Lift it.", "to": "ask", "unless": [{"key": key}], "effects": [roll(json!({}))]});
    assert_eq!(loop_warnings(own, false), want, "an unless on the roll's own key");
    // nor a set_flag of the roll's own key: the roll overwrites it with false on a failure
    let own_set = json!({"text": "Lift it.", "to": "ask", "unless": [{"key": key}], "effects": [{"effect": "set_flag", "key": key}, roll(json!({}))]});
    assert_eq!(loop_warnings(own_set, false), want, "a set_flag of the roll's own key");
}

#[test]
fn a_guarded_or_cycle_free_roll_choice_does_not_warn() {
    let set = json!({"text": "Lift it.", "to": "ask", "unless": [{"key": "tried_beam"}], "effects": [roll(json!({})), {"effect": "set_flag", "key": "tried_beam"}]});
    assert!(loop_warnings(set, false).is_empty(), "an unless the choice's own set_flag sets (the give guard)");
    assert!(loop_warnings(json!({"text": "Lift it.", "to": "bye", "effects": [roll(json!({}))]}), false).is_empty(), "no cycle: picked once");
}

// ---- R4-2 (FB-143) AUDIT-05: a derived stat name counts only where the world's ruleset keeps the row --------------

/// Worlds `w` (classic) and `t` (ruleset `timeline`); the derived rows `grit` (every ruleset) and `tl_grit` (timeline
/// only, by its `rulesets`).
fn ruleset_view() -> View {
    let hero = |w: &str| json!([{"id": "hero", "name": "Hero", "role": "party", "_world": w, "_qid": format!("{w}:hero")}]);
    let worlds = json!({
        "w": {"manifest": {"id": "w", "name": "W"}, "own": {"characters": hero("w")}},
        "t": {"manifest": {"id": "t", "name": "T", "ruleset": "timeline"}, "own": {"characters": hero("t")}}
    });
    let catalog = json!({
        "rulesets": [{"id": "timeline", "name": "Timeline"}],
        "derived_stats": [
            {"id": "grit", "name": "Grit", "formula": "CONS * 2 + 1"},
            {"id": "tl_grit", "name": "Timeline Grit", "formula": "CONS * 2 + 1", "rulesets": ["timeline"]}
        ]
    });
    View::new(&catalog, &worlds, &json!({}), Default::default())
}

#[test]
fn a_derived_stat_left_out_of_the_worlds_ruleset_is_refused() {
    let v = ruleset_view();
    let roll_errors = |w: &str, stat: &str| dialogue::list_errors(&v, w, &json!([roll(json!({"stat": stat, "dc": 20}))]), LABEL, EffectList::Choice);
    let cond_errors = |w: &str, stat: &str| dialogue::conditions_errors(&v, w, &json!([{"kind": "stat", "stat": stat, "min": 20}]), "Requires");
    for w in ["w", "t"] {
        assert!(roll_errors(w, "grit").is_empty() && cond_errors(w, "grit").is_empty(), "{w}: a row of every ruleset");
    }
    assert!(roll_errors("t", "tl_grit").is_empty() && cond_errors("t", "tl_grit").is_empty(), "the timeline world reads its row");
    let roll_bad = roll_errors("w", "tl_grit");
    assert_eq!(roll_bad.len(), 1, "{roll_bad:?}");
    assert!(roll_bad[0].contains("stat 'tl_grit' is not one of STR") && roll_bad[0].ends_with("derived stat id of the world's ruleset)."), "{roll_bad:?}");
    let cond_bad = cond_errors("w", "tl_grit");
    assert_eq!(cond_bad.len(), 1, "{cond_bad:?}");
    assert!(cond_bad[0].contains("stat 'tl_grit' is not one of STR"), "{cond_bad:?}");
}
