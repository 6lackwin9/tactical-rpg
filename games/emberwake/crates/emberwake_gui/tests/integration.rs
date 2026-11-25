//! Consolidated integration test harness for `emberwake_gui`.
//!
//! Groups 3 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: ui_bot.rs.

#[path = "suite/frame_tuner.rs"]
mod frame_tuner;
#[path = "suite/hud_icons.rs"]
mod hud_icons;
#[path = "suite/theme.rs"]
mod theme;
