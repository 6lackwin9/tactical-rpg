//! Consolidated integration test harness for `emberwake_content`.
//!
//! Groups 10 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: checks_golden.rs.

#[path = "suite/art_looks.rs"]
mod art_looks;
#[path = "suite/asset_checks.rs"]
mod asset_checks;
#[path = "suite/classic_ruleset.rs"]
mod classic_ruleset;
#[path = "suite/face_sidecars.rs"]
mod face_sidecars;
#[path = "suite/family_table.rs"]
mod family_table;
#[path = "suite/indoor_rooms.rs"]
mod indoor_rooms;
#[path = "suite/manifest_resave.rs"]
mod manifest_resave;
#[path = "suite/prop_checks.rs"]
mod prop_checks;
#[path = "suite/reward_checks.rs"]
mod reward_checks;
#[path = "suite/ruleset_switches.rs"]
mod ruleset_switches;
