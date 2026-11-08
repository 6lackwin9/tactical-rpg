//! CLN-45: the switches the round intent rule used to bundle are ruleset row fields (decision "Godot parity retired
//! and cleanup pass 3 answers" Q-B2). The shipped rows play as before whether or not they set the fields yet: the
//! `timeline` row (with `round_intent`) reads the round intent rule's choices and the `classic` row the legacy ones
//! (the built-in classic values). The seed `data/seeds/cln45.json` sets the fields to exactly those values: applied to
//! a scratch copy of the data, every row reads as before bar the rules HOOKS1-A appended later to `timeline`, and the
//! file re-saves byte for byte; `data/seeds/hooks1a.json` then `data/seeds/s22s3.json` (S2.2-S3: `poise_mode` pips)
//! over it give the shipped row back. The unit deck orders `data/seeds/w24k1b.json` set by derived intent rows (valid
//! only under `field_intent`) are set aside during the replay and re-applied after `w24s1.json`.

use aegis_core::json::{self, Dict};
use emberwake_content::{source, store};
use emberwake_rules::ruleset::{self, Chase, PoiseMode, RestAp, Ruleset, AI_TACTICS, ARENA_HAZARDS, BY_RULES_FIELDS, CLASSIC, EXECUTE, FORCED_MOVE, GROUND_HAZARDS, HP_COST, IGNORE_GUARD, ROUND_INTENT, TIMELINE, TRIGGERS, UNIT_METERS};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// HOOKS1_TASKS 2.4: what `rulesets.timeline.rules` appends, in its order.
const HOOKS1_RULES: [&str; 9] = [FORCED_MOVE, GROUND_HAZARDS, ARENA_HAZARDS, TRIGGERS, UNIT_METERS, HP_COST, IGNORE_GUARD, AI_TACTICS, EXECUTE];

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn switches(r: &Ruleset) -> (RestAp, Chase, bool, bool, bool) {
    (r.rest_ap(), r.chase(), r.root_stops_chase(), r.switch_target_step(), r.stances())
}

/// The loaded ruleset rows of the data folder `dir`, by id.
fn rows(dir: &Path) -> Vec<(String, Ruleset)> {
    let v = store::view(dir);
    v.entries(ruleset::KIND).iter().map(|e| (json::s(e, "id"), Ruleset::from_row(e))).collect()
}

/// The ruleset rows the seed `name` upserts, as given.
fn seed_entries(name: &str) -> Vec<Dict> {
    seed_rows(name, ruleset::KIND)
}

/// The rows of kind `kind` the seed `name` upserts, as given.
fn seed_rows(name: &str, kind: &str) -> Vec<Dict> {
    let text = std::fs::read_to_string(data_dir().join("seeds").join(name)).expect("the seed");
    let seed: Value = serde_json::from_str(&text).expect("seed JSON");
    json::arr(&seed["steps"]).iter().map(json::dict).filter(|s| json::s(s, "kind") == kind).map(|s| json::dget(s, "entry").clone()).collect()
}

/// The unit decks the seed `w24k1b.json` (A4.2-K1b) re-ordered by derived intent rows (a `field` list, a ring in
/// `guard`): those orders are valid only under `field_intent`, so while the pre-W2-4 ruleset seeds replay they are
/// set aside (an empty order, valid under either reading) and the seed puts them back after `w24s1.json`.
const DERIVED_ORDERS_SEED: &str = "w24k1b.json";

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_owned();
        if p.is_dir() {
            copy_dir(&p, &to.join(name));
        } else {
            std::fs::copy(&p, to.join(name)).unwrap();
        }
    }
}

#[test]
fn the_shipped_rows_play_as_before() {
    let shipped = rows(&data_dir());
    assert!(!shipped.is_empty(), "the shipped rulesets");
    let intent_choices = (RestAp::RoundEnd, Chase::EverySkill, true, false, false);
    for (id, r) in &shipped {
        let want = if r.has(ROUND_INTENT) { intent_choices } else { switches(&Ruleset::classic()) };
        assert_eq!(switches(r), want, "ruleset '{id}'");
    }
    let classic = shipped.iter().find(|(id, _)| id == CLASSIC).map(|(_, r)| r).expect("the classic row");
    assert_eq!(classic, &Ruleset::classic(), "classic battles unchanged");
}

#[test]
fn the_seed_sets_the_values_the_rows_read_now() {
    let before = rows(&data_dir());
    let entries = seed_entries("cln45.json");
    assert_eq!(entries.len(), before.len(), "one upsert per shipped row");
    for e in &entries {
        assert!(BY_RULES_FIELDS.iter().all(|k| !json::s(e, k).is_empty()), "the seed sets every switch of '{}'", json::s(e, "id"));
    }
    let tmp = aegis_core::scratch::Scratch::vacant("emberwake_cln45_seed");
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    // the units the derived-row seed re-ordered, as they stood before it as far as the ruleset rows care: no order
    let shipped_units = store::view(&data_dir());
    let reordered = seed_rows(DERIVED_ORDERS_SEED, "units");
    assert!(!reordered.is_empty(), "the derived-row seed re-orders unit decks");
    for u in &reordered {
        let id = json::s(u, "id");
        let mut e = shipped_units.get("units", &id).clone();
        assert_eq!(e.get("deck"), u.get("deck"), "unit '{id}' ships with the deck the seed leaves");
        if let Some(Value::Object(deck)) = e.get_mut("deck") {
            deck.shift_remove("order");
        }
        store::upsert(&tmp, None, "units", &e, None).unwrap_or_else(|err| panic!("unit '{id}' without its order: {err:?}"));
    }
    for e in &entries {
        store::upsert(&tmp, None, ruleset::KIND, e, None).unwrap_or_else(|err| panic!("seed row '{}': {err:?}", json::s(e, "id")));
    }
    let mut want = before.clone();
    for (id, r) in want.iter_mut().filter(|(id, _)| id == TIMELINE) {
        let (kept, added) = r.rules.split_at(r.rules.len() - HOOKS1_RULES.len());
        assert_eq!(added, HOOKS1_RULES, "ruleset '{id}' ends with what HOOKS1 appends");
        // W2-4 slice 1 (FB-106): the seed w24s1 added field_intent later
        r.rules = kept.iter().filter(|x| **x != ruleset::FIELD_INTENT).cloned().collect();
        // S2.2-S3: the seed s22s3 set poise_mode pips later
        r.numbers.poise_mode = PoiseMode::Bar;
        // R1-BAL H6 (FB-98): the seed r1bal set stagger_windup_mult 2.0 -> 1.0 later
        r.numbers.stagger_windup_mult = 2.0;
        // A1.2-S1 (FB-55): the seed a12s1 set item_ap 0 later
        r.numbers.item_ap = 1;
    }
    assert_eq!(rows(&tmp), want, "every row reads as before, bar the rules HOOKS1 appended later");
    let file = store::file_of(&tmp, None, ruleset::KIND);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(BY_RULES_FIELDS.iter().all(|k| text.contains(&format!("\"{k}\""))), "the switches are written");
    assert_eq!(store::resave_text(&tmp, None, ruleset::KIND).unwrap(), text, "the seeded file re-saves byte for byte");
    assert!(source::load(&tmp).errors.iter().all(|e| !e.contains("rulesets.json")), "the seeded rows load clean");
    for e in seed_entries("hooks1a.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("HOOKS1-A row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("s22s3.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("S2.2-S3 row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("r1bal.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("R1-BAL row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("a12s1.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("A1.2-S1 row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("w24s1.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("W2-4 slice 1 row '{}': {err:?}", json::s(&e, "id")));
    }
    assert_eq!(rows(&tmp), before, "the HOOKS1-A, S2.2-S3, R1-BAL, A1.2-S1 and W2-4 slice 1 rows over it give the shipped row");
    // A4.2-K1b: under field_intent the derived-row seed's orders are valid again and give the shipped units back
    for u in &reordered {
        store::upsert(&tmp, None, "units", u, None).unwrap_or_else(|err| panic!("W2-4 slice 6a unit '{}': {err:?}", json::s(u, "id")));
    }
    let seeded = store::view(&tmp);
    for u in &reordered {
        let id = json::s(u, "id");
        assert_eq!(seeded.get("units", &id), shipped_units.get("units", &id), "unit '{id}' as shipped");
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
