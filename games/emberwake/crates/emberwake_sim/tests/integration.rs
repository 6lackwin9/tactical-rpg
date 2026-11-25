//! Consolidated integration test harness for `emberwake_sim`.
//!
//! Groups 3 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: sim_golden.rs.

#[path = "suite/config.rs"]
mod config;
#[path = "suite/rules.rs"]
mod rules;
#[path = "suite/world_feel.rs"]
mod world_feel;
