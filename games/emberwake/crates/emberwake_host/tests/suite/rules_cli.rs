//! R4-2 (3), X5 on the shipped data: `aegis rules` (the kit's tweak report over the game's ruleset rows, built as the CLI
//! builds it) notes every shipped row "ok" (no error, no warning), and `aegis rules --contract` on the game's schema is
//! the kit's contract golden line for line (`rpgkit_host/tests/fixtures/rules_contract_golden.json`), so the game adds
//! no contract of its own. The shipped `content check` is `emberwake_world/tests/store.rs`
//! (`shipped_data_checks_clean`).

use emberwake_rules::ruleset::{self, Ruleset};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fields() -> &'static [Value] {
    aegis_content::schema::fields(emberwake_content::source::schema(), "content", ruleset::KIND)
}

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn the_rules_report_notes_every_shipped_row_ok() {
    let (d, problems) = emberwake_world::Data::open(&data_dir()).expect("the shipped data opens");
    assert!(problems.is_empty(), "{problems:?}");
    let rows: Vec<Ruleset> = d.cat.rulesets.values().map(Ruleset::from_row).collect();
    assert!(!rows.is_empty(), "the shipped data has ruleset rows");
    let lines = rpgkit_host::rules::report(fields(), &rows);
    let at = lines.iter().position(|l| l == "Rows:").expect("the Rows section");
    let notes: Vec<&String> = lines[at + 1..].iter().filter(|l| l.starts_with("    ")).collect();
    assert_eq!(notes.len(), rows.len(), "one note line per row (ok): {:?}", &lines[at..]);
    assert!(notes.iter().all(|l| l.as_str() == "    ok"), "{:?}", &lines[at..]);
}

#[test]
fn the_games_rules_contract_is_the_kits_golden() {
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../kits/rpg/crates/rpgkit_host/tests/fixtures/rules_contract_golden.json");
    let golden: Value = serde_json::from_str(&std::fs::read_to_string(&golden).expect("the kit's contract golden")).unwrap();
    let want: Vec<String> = golden["contract"].as_array().expect("contract lines").iter().map(|v| v.as_str().unwrap_or_default().to_string()).collect();
    assert_eq!(rpgkit_host::rules::contract(fields()), want);
}
