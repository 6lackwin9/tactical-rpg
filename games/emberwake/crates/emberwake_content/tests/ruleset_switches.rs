//! CLN-45: the switches the round intent rule used to bundle are ruleset row fields (decision "Godot parity retired
//! and cleanup pass 3 answers" Q-B2). The shipped rows play as before whether or not they set the fields yet: the
//! `timeline` row (with `round_intent`) reads the round intent rule's choices and the `classic` row the legacy ones
//! (the built-in classic values). The seed `data/seeds/cln45.json` sets the fields to exactly those values: applied to
//! a scratch copy of the data, every row reads as before and the file re-saves byte for byte.

use aegis_core::json::{self, Dict};
use emberwake_content::{source, store};
use emberwake_rules::ruleset::{self, Chase, RestAp, Ruleset, BY_RULES_FIELDS, CLASSIC, ROUND_INTENT};
use serde_json::Value;
use std::path::{Path, PathBuf};

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

/// The rows the seed upserts, as given.
fn seed_entries() -> Vec<Dict> {
    let text = std::fs::read_to_string(data_dir().join("seeds/cln45.json")).expect("the CLN-45 seed");
    let seed: Value = serde_json::from_str(&text).expect("seed JSON");
    json::arr(&seed["steps"]).iter().map(json::dict).filter(|s| json::s(s, "kind") == ruleset::KIND).map(|s| json::dget(s, "entry").clone()).collect()
}

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
    let entries = seed_entries();
    assert_eq!(entries.len(), before.len(), "one upsert per shipped row");
    for e in &entries {
        assert!(BY_RULES_FIELDS.iter().all(|k| !json::s(e, k).is_empty()), "the seed sets every switch of '{}'", json::s(e, "id"));
    }
    let tmp = std::env::temp_dir().join(format!("emberwake_cln45_seed_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    for e in &entries {
        store::upsert(&tmp, None, ruleset::KIND, e, None).unwrap_or_else(|err| panic!("seed row '{}': {err:?}", json::s(e, "id")));
    }
    assert_eq!(rows(&tmp), before, "every row reads as before");
    let file = store::file_of(&tmp, None, ruleset::KIND);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(BY_RULES_FIELDS.iter().all(|k| text.contains(&format!("\"{k}\""))), "the switches are written");
    assert_eq!(store::resave_text(&tmp, None, ruleset::KIND).unwrap(), text, "the seeded file re-saves byte for byte");
    assert!(source::load(&tmp).errors.iter().all(|e| !e.contains("rulesets.json")), "the seeded rows load clean");
    let _ = std::fs::remove_dir_all(&tmp);
}
