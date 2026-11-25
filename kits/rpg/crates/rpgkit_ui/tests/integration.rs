//! Consolidated integration test harness for `rpgkit_ui`.
//!
//! Groups 7 integration suites under one binary to avoid redundant link steps.
//! All integration suites run in this single binary.

#[path = "suite/pips.rs"]
mod pips;
#[path = "suite/pips_native.rs"]
mod pips_native;
#[path = "suite/portrait_fx.rs"]
mod portrait_fx;
#[path = "suite/portrait_overlays.rs"]
mod portrait_overlays;
#[path = "suite/portrait_page.rs"]
mod portrait_page;
#[path = "suite/theme_panes.rs"]
mod theme_panes;
#[path = "suite/vignette.rs"]
mod vignette;
