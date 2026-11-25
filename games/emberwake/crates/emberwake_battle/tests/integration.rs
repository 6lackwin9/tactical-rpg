//! Consolidated integration test harness for `emberwake_battle`.
//!
//! Groups 4 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: hit_forecast_timeline.rs, timeline_golden.rs.

#[path = "suite/catalyst_pulse.rs"]
mod catalyst_pulse;
#[path = "suite/poise_pips_timeline.rs"]
mod poise_pips_timeline;
#[path = "suite/step_events.rs"]
mod step_events;
#[path = "suite/terrain_rows.rs"]
mod terrain_rows;
