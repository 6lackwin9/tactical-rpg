//! Consolidated integration test harness for `emberwake_rules`.
//!
//! Groups 4 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: rules_golden.rs.

#[path = "suite/hit_modifiers.rs"]
mod hit_modifiers;
#[path = "suite/ir21_intent.rs"]
mod ir21_intent;
#[path = "suite/passive_grants.rs"]
mod passive_grants;
#[path = "suite/unit_budget.rs"]
mod unit_budget;
