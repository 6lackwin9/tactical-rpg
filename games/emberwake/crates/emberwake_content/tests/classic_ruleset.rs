//! CLN-44: the classic ruleset was a row of the game's `rulesets` (decision "Godot parity retired and cleanup pass 3
//! answers" Q-B1). The seed `data/seeds/cln44.json` authored it with the built-in classic values exactly (no rules,
//! `melee_diagonal` none, the default numbers), so classic battles did not change when the row landed. PR5-52
//! (RULESET-MODES1, FB-223, `P-ARCH-02`): the game ships one ruleset row (`timeline`); `classic` is the kit's built-in
//! preset, which the worlds that name no ruleset (the Ashlands, the Iron Reaches, the Tidewrack) resolve as before.

use aegis_core::json::{self, Dict};
use emberwake_content::{source, store};
use emberwake_rules::ruleset::{self, Ruleset, CLASSIC};
use serde_json::Value;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// The `classic` entry the seed upserts, normalized as the writer stores it.
fn seed_entry() -> Dict {
    let text = std::fs::read_to_string(data_dir().join("seeds/cln44.json")).expect("the CLN-44 seed");
    let seed: Value = serde_json::from_str(&text).expect("seed JSON");
    let step = json::arr(&seed["steps"]).iter().map(json::dict).find(|s| json::s(s, "kind") == ruleset::KIND).expect("a rulesets step").clone();
    let entry = json::dget(&step, "entry").clone();
    aegis_content::schema::normalize(aegis_content::schema::fields(source::schema(), "content", ruleset::KIND), &entry, false)
}

#[test]
fn the_seed_row_reads_as_the_built_in_classic_values() {
    let row = seed_entry();
    assert_eq!(json::s(&row, "id"), CLASSIC);
    assert_eq!(Ruleset::from_row(&row), Ruleset::classic(), "the row's rules and numbers are the built-in classic ones");
    assert_eq!(emberwake_content::check::ruleset::warnings(&row), Vec::<String>::new());
}

/// PR5-52: the shipped catalog holds the timeline row only; classic resolves to the kit's built-in preset (as the
/// shipped row read), and each shipped world resolves its ruleset: a world naming none plays classic, with no error.
#[test]
fn the_game_ships_one_ruleset_and_classic_is_the_kit_preset() {
    let v = store::view(&data_dir());
    let ids: Vec<String> = v.entries(ruleset::KIND).iter().map(|e| json::s(e, "id")).collect();
    assert_eq!(ids, [ruleset::TIMELINE], "one shipped ruleset");
    let cat = emberwake_data::Catalog::from_value(&source::load(&data_dir()).catalog);
    assert_eq!(Ruleset::of(&cat, CLASSIC), Ok(Ruleset::classic()), "classic: the preset");
    assert!(Ruleset::of(&cat, ruleset::TIMELINE).is_ok_and(|r| r.has(ruleset::ROUND_INTENT)), "timeline: the row");
    let mut classic_worlds = Vec::new();
    for e in std::fs::read_dir(data_dir().join("worlds")).unwrap() {
        let dir = e.unwrap().path();
        let manifest: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("world.json")).unwrap()).unwrap();
        let id = ruleset::world_ruleset([json::dict(&manifest)]);
        let r = Ruleset::of(&cat, &id).unwrap_or_else(|err| panic!("{}: {err}", dir.display()));
        if r.is_classic() {
            classic_worlds.push(json::s(json::dict(&manifest), "id"));
        }
    }
    assert!(classic_worlds.iter().any(|w| w == "ashlands"), "the Ashlands name no ruleset and play classic: {classic_worlds:?}");
}

/// BASERULES1-M: the register's metadata is additive: every shipped ruleset row still loads, and the default rows
/// (classic, timeline) raise no metadata warning (a test row may: it is printed).
#[test]
fn the_shipped_rows_pass_the_register_metadata() {
    let v = store::view(&data_dir());
    let rows = v.entries(ruleset::KIND);
    assert!(source::load(&data_dir()).errors.iter().all(|e| !e.contains("rulesets.json")), "every shipped row loads");
    for row in rows {
        let notes = Ruleset::from_row(row).notes();
        let id = json::s(row, "id");
        assert!(notes.errors.is_empty(), "{id}: {:?}", notes.errors);
        if id == CLASSIC || id == ruleset::TIMELINE {
            assert_eq!(emberwake_content::check::ruleset::warnings(row), Vec::<String>::new(), "{id}");
        } else if !notes.warnings.is_empty() {
            eprintln!("rulesets:{id}: {:?}", notes.warnings);
        }
    }
}
