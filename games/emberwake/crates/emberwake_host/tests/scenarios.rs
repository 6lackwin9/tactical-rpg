//! HEADLESS1: the game's scenario files parse, so a change to the format or a bad edit fails the gate. No game starts here
//! (running them is `tools/fc.sh scenario all`).

use aegis_cli::scenario::{check_files, files_in};
use std::path::Path;

#[test]
fn the_shipped_scenarios_parse() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");
    let files = files_in(&folder);
    assert!(!files.is_empty(), "no scenario files in {}", folder.display());
    let problems = check_files(&files);
    assert!(problems.is_empty(), "{problems:#?}");
}
