//! Consolidated integration test harness for `rpgkit_host`.
//!
//! Groups 10 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: rules_report.rs.

#[path = "suite/battle_talk.rs"]
mod battle_talk;
#[path = "suite/party_retreat.rs"]
mod party_retreat;
#[path = "suite/pick_requests.rs"]
mod pick_requests;
#[path = "suite/plan_badges.rs"]
mod plan_badges;
#[path = "suite/round_intent_requests.rs"]
mod round_intent_requests;
#[path = "suite/script_round.rs"]
mod script_round;
#[path = "suite/stay_pick.rs"]
mod stay_pick;
#[path = "suite/switch_retired.rs"]
mod switch_retired;
#[path = "suite/unmark_turned.rs"]
mod unmark_turned;
#[path = "suite/wear_keys.rs"]
mod wear_keys;
