//! CLN-44: the classic ruleset is a row of the game's `rulesets` (decision "Godot parity retired and cleanup pass 3
//! answers" Q-B1). The seed `data/seeds/cln44.json` authors it with the built-in classic values exactly (no rules,
//! `melee_diagonal` none, the default numbers), so classic battles do not change when the row lands; once the shipped
//! catalog has the row, it reads as those values too.

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

#[test]
fn a_shipped_classic_row_reads_as_the_built_in_classic_values() {
    let v = store::view(&data_dir());
    let row = v.entries(ruleset::KIND).iter().find(|e| json::s(e, "id") == CLASSIC).expect("the shipped classic row (seed cln44.json)");
    assert_eq!(Ruleset::from_row(row), Ruleset::classic(), "classic battles unchanged");
}
