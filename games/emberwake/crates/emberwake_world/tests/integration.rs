//! Consolidated integration test harness for `emberwake_world`.
//!
//! Groups 41 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: journey.rs, phasefold.rs.

#[path = "suite/act1_hooks.rs"]
mod act1_hooks;
#[path = "suite/act1_story.rs"]
mod act1_story;
#[path = "suite/auto_plan_cages.rs"]
mod auto_plan_cages;
#[path = "suite/big_units.rs"]
mod big_units;
#[path = "suite/board_rec_level.rs"]
mod board_rec_level;
#[path = "suite/bunk_talks.rs"]
mod bunk_talks;
#[path = "suite/cast1_probes.rs"]
mod cast1_probes;
#[path = "suite/class_keystone.rs"]
mod class_keystone;
#[path = "suite/custom_gear_family.rs"]
mod custom_gear_family;
#[path = "suite/custom_terrain.rs"]
mod custom_terrain;
#[path = "suite/deck_grants.rs"]
mod deck_grants;
#[path = "suite/default_decks.rs"]
mod default_decks;
#[path = "suite/dojo.rs"]
mod dojo;
#[path = "suite/fight1_hint.rs"]
mod fight1_hint;
#[path = "suite/fight2_hint.rs"]
mod fight2_hint;
#[path = "suite/generated_decor.rs"]
mod generated_decor;
#[path = "suite/history.rs"]
mod history;
#[path = "suite/intents.rs"]
mod intents;
#[path = "suite/item_text.rs"]
mod item_text;
#[path = "suite/job_fight_repair.rs"]
mod job_fight_repair;
#[path = "suite/no_trade_lines.rs"]
mod no_trade_lines;
#[path = "suite/npc_services.rs"]
mod npc_services;
#[path = "suite/patrol_pacing.rs"]
mod patrol_pacing;
#[path = "suite/relic_signatures.rs"]
mod relic_signatures;
#[path = "suite/report.rs"]
mod report;
#[path = "suite/rewards.rs"]
mod rewards;
#[path = "suite/rulesets.rs"]
mod rulesets;
#[path = "suite/save_aliases.rs"]
mod save_aliases;
#[path = "suite/shop_trade.rs"]
mod shop_trade;
#[path = "suite/skill_ranks.rs"]
mod skill_ranks;
#[path = "suite/skill_text.rs"]
mod skill_text;
#[path = "suite/store.rs"]
mod store;
#[path = "suite/strict_loadouts.rs"]
mod strict_loadouts;
#[path = "suite/sunk_ward_act1_marrow.rs"]
mod sunk_ward_act1_marrow;
#[path = "suite/sunk_ward_maps.rs"]
mod sunk_ward_maps;
#[path = "suite/sunk_ward_roster.rs"]
mod sunk_ward_roster;
#[path = "suite/sunk_ward_text.rs"]
mod sunk_ward_text;
#[path = "suite/sweep_telegraph.rs"]
mod sweep_telegraph;
#[path = "suite/tallow_upkeep.rs"]
mod tallow_upkeep;
#[path = "suite/techniques_grafts.rs"]
mod techniques_grafts;
#[path = "suite/unit_builder.rs"]
mod unit_builder;
