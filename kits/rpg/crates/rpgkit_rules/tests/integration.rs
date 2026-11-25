//! Consolidated integration test harness for `rpgkit_rules`.
//!
//! Groups 30 integration suites under one binary to avoid redundant link steps.
//! All integration suites run in this single binary.

#[path = "suite/auto_spend.rs"]
mod auto_spend;
#[path = "suite/body_grafts.rs"]
mod body_grafts;
#[path = "suite/carry.rs"]
mod carry;
#[path = "suite/class_basics.rs"]
mod class_basics;
#[path = "suite/class_keystone.rs"]
mod class_keystone;
#[path = "suite/class_passives.rs"]
mod class_passives;
#[path = "suite/crit_rate.rs"]
mod crit_rate;
#[path = "suite/damage_pipeline.rs"]
mod damage_pipeline;
#[path = "suite/derived.rs"]
mod derived;
#[path = "suite/ir21_intent.rs"]
mod ir21_intent;
#[path = "suite/item_grants.rs"]
mod item_grants;
#[path = "suite/item_values.rs"]
mod item_values;
#[path = "suite/locked.rs"]
mod locked;
#[path = "suite/member_grants.rs"]
mod member_grants;
#[path = "suite/move_rate.rs"]
mod move_rate;
#[path = "suite/pools.rs"]
mod pools;
#[path = "suite/rating_status_reads.rs"]
mod rating_status_reads;
#[path = "suite/rule_metadata.rs"]
mod rule_metadata;
#[path = "suite/slot_rules.rs"]
mod slot_rules;
#[path = "suite/stage_passives.rs"]
mod stage_passives;
#[path = "suite/status_reads.rs"]
mod status_reads;
#[path = "suite/status_table.rs"]
mod status_table;
#[path = "suite/strict_gear.rs"]
mod strict_gear;
#[path = "suite/take_damage_pipeline.rs"]
mod take_damage_pipeline;
#[path = "suite/tier_factors.rs"]
mod tier_factors;
#[path = "suite/timeline_statuses.rs"]
mod timeline_statuses;
#[path = "suite/unit_budget.rs"]
mod unit_budget;
#[path = "suite/unit_templates.rs"]
mod unit_templates;
#[path = "suite/weapon_switch.rs"]
mod weapon_switch;
#[path = "suite/wield_penalty.rs"]
mod wield_penalty;
