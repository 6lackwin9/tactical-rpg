//! Consolidated integration test harness for `rpgkit_sim`.
//!
//! Groups 17 integration suites under one binary to avoid redundant link steps.
//! All integration suites run in this single binary.

#[path = "suite/channel_counts.rs"]
mod channel_counts;
#[path = "suite/classes.rs"]
mod classes;
#[path = "suite/config.rs"]
mod config;
#[path = "suite/deadlines.rs"]
mod deadlines;
#[path = "suite/default_list.rs"]
mod default_list;
#[path = "suite/dulled.rs"]
mod dulled;
#[path = "suite/feel.rs"]
mod feel;
#[path = "suite/feel_counts.rs"]
mod feel_counts;
#[path = "suite/feel_world.rs"]
mod feel_world;
#[path = "suite/primary_stat.rs"]
mod primary_stat;
#[path = "suite/ruleset_args.rs"]
mod ruleset_args;
#[path = "suite/seed_args.rs"]
mod seed_args;
#[path = "suite/stage.rs"]
mod stage;
#[path = "suite/story_run.rs"]
mod story_run;
#[path = "suite/whatif.rs"]
mod whatif;
#[path = "suite/world_campaign.rs"]
mod world_campaign;
#[path = "suite/world_characters.rs"]
mod world_characters;
