//! Consolidated integration test harness for `rpgkit_world`.
//!
//! Groups 68 integration suites under one binary to avoid redundant link steps.
//! All integration suites run in this single binary.

#[path = "suite/battle_hooks.rs"]
mod battle_hooks;
#[path = "suite/board.rs"]
mod board;
#[path = "suite/body_slots.rs"]
mod body_slots;
#[path = "suite/bunk_lockers.rs"]
mod bunk_lockers;
#[path = "suite/bunk_upgrade_props.rs"]
mod bunk_upgrade_props;
#[path = "suite/bunks.rs"]
mod bunks;
#[path = "suite/campaign.rs"]
mod campaign;
#[path = "suite/campaign_days.rs"]
mod campaign_days;
#[path = "suite/campaign_hooks.rs"]
mod campaign_hooks;
#[path = "suite/campaign_triggers.rs"]
mod campaign_triggers;
#[path = "suite/campaign_view.rs"]
mod campaign_view;
#[path = "suite/class_overlay.rs"]
mod class_overlay;
#[path = "suite/craft_recipes.rs"]
mod craft_recipes;
#[path = "suite/data_dir.rs"]
mod data_dir;
#[path = "suite/deck.rs"]
mod deck;
#[path = "suite/deck_rows.rs"]
mod deck_rows;
#[path = "suite/dialogue_roll.rs"]
mod dialogue_roll;
#[path = "suite/dialogue_threshold.rs"]
mod dialogue_threshold;
#[path = "suite/endings.rs"]
mod endings;
#[path = "suite/explore_leader.rs"]
mod explore_leader;
#[path = "suite/expressions.rs"]
mod expressions;
#[path = "suite/facilities.rs"]
mod facilities;
#[path = "suite/family_looks.rs"]
mod family_looks;
#[path = "suite/flag_skills.rs"]
mod flag_skills;
#[path = "suite/graft_install_effect.rs"]
mod graft_install_effect;
#[path = "suite/ground_walking.rs"]
mod ground_walking;
#[path = "suite/guest_survivors.rs"]
mod guest_survivors;
#[path = "suite/item_uses.rs"]
mod item_uses;
#[path = "suite/job_fights.rs"]
mod job_fights;
#[path = "suite/job_guidance.rs"]
mod job_guidance;
#[path = "suite/loadout_sanitize.rs"]
mod loadout_sanitize;
#[path = "suite/map_reveal.rs"]
mod map_reveal;
#[path = "suite/mastery.rs"]
mod mastery;
#[path = "suite/party_ops.rs"]
mod party_ops;
#[path = "suite/plan_reveal.rs"]
mod plan_reveal;
#[path = "suite/points_of_interest.rs"]
mod points_of_interest;
#[path = "suite/pouch_stacks.rs"]
mod pouch_stacks;
#[path = "suite/purse_caps.rs"]
mod purse_caps;
#[path = "suite/recipes.rs"]
mod recipes;
#[path = "suite/road_notice.rs"]
mod road_notice;
#[path = "suite/roster.rs"]
mod roster;
#[path = "suite/rule_effect_gates.rs"]
mod rule_effect_gates;
#[path = "suite/save_gates.rs"]
mod save_gates;
#[path = "suite/save_versions.rs"]
mod save_versions;
#[path = "suite/scene_must_choose.rs"]
mod scene_must_choose;
#[path = "suite/shop_discount.rs"]
mod shop_discount;
#[path = "suite/sieges.rs"]
mod sieges;
#[path = "suite/start_worlds.rs"]
mod start_worlds;
#[path = "suite/story.rs"]
mod story;
#[path = "suite/story_music.rs"]
mod story_music;
#[path = "suite/story_prep.rs"]
mod story_prep;
#[path = "suite/strict_wear.rs"]
mod strict_wear;
#[path = "suite/supplies_grants.rs"]
mod supplies_grants;
#[path = "suite/talks.rs"]
mod talks;
#[path = "suite/temperament.rs"]
mod temperament;
#[path = "suite/terms.rs"]
mod terms;
#[path = "suite/travel_modes.rs"]
mod travel_modes;
#[path = "suite/travel_paths.rs"]
mod travel_paths;
#[path = "suite/turn_in_lines.rs"]
mod turn_in_lines;
#[path = "suite/turn_in_tick.rs"]
mod turn_in_tick;
#[path = "suite/unit_builder.rs"]
mod unit_builder;
#[path = "suite/unit_templates.rs"]
mod unit_templates;
#[path = "suite/unit_tracks.rs"]
mod unit_tracks;
#[path = "suite/weapon_instances.rs"]
mod weapon_instances;
#[path = "suite/weapon_recipes.rs"]
mod weapon_recipes;
#[path = "suite/wear_drills.rs"]
mod wear_drills;
#[path = "suite/world_fight.rs"]
mod world_fight;
#[path = "suite/world_ruleset.rs"]
mod world_ruleset;
