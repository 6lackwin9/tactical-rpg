//! Consolidated integration test harness for `emberwake_host`.
//!
//! Groups 9 integration suites under one binary to avoid redundant link steps.
//! All integration suites run in this single binary.

#[path = "suite/auto_driver.rs"]
mod auto_driver;
#[path = "suite/content_world.rs"]
mod content_world;
#[path = "suite/replay.rs"]
mod replay;
#[path = "suite/rules_cli.rs"]
mod rules_cli;
#[path = "suite/ruleset_requests.rs"]
mod ruleset_requests;
#[path = "suite/scenarios.rs"]
mod scenarios;
#[path = "suite/web_bundle.rs"]
mod web_bundle;
#[path = "suite/world_battle.rs"]
mod world_battle;
#[path = "suite/world_map_layout_cli.rs"]
mod world_map_layout_cli;
