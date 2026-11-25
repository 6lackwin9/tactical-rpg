//! Consolidated integration test harness for `rpgkit_battle`.
//!
//! Groups 89 integration suites under one binary to avoid redundant link steps.
//! Standalone binaries kept for process-global state or golden discovery: intent_ai.rs.

#[path = "suite/ai_rating_scorer.rs"]
mod ai_rating_scorer;
#[path = "suite/ai_rows.rs"]
mod ai_rows;
#[path = "suite/ai_status_reads.rs"]
mod ai_status_reads;
#[path = "suite/area_affects.rs"]
mod area_affects;
#[path = "suite/area_shapes.rs"]
mod area_shapes;
#[path = "suite/area_tell.rs"]
mod area_tell;
#[path = "suite/arena_hazards.rs"]
mod arena_hazards;
#[path = "suite/attack_lock.rs"]
mod attack_lock;
#[path = "suite/auto_plan_fixes.rs"]
mod auto_plan_fixes;
#[path = "suite/baseline_enrage.rs"]
mod baseline_enrage;
#[path = "suite/battle_events.rs"]
mod battle_events;
#[path = "suite/battle_hooks.rs"]
mod battle_hooks;
#[path = "suite/battle_skill_grants.rs"]
mod battle_skill_grants;
#[path = "suite/battle_talk.rs"]
mod battle_talk;
#[path = "suite/big_units.rs"]
mod big_units;
#[path = "suite/bleed_out.rs"]
mod bleed_out;
#[path = "suite/channel.rs"]
mod channel;
#[path = "suite/class_basics.rs"]
mod class_basics;
#[path = "suite/data_pool.rs"]
mod data_pool;
#[path = "suite/deck.rs"]
mod deck;
#[path = "suite/deck_unlock.rs"]
mod deck_unlock;
#[path = "suite/dojo_member_grants.rs"]
mod dojo_member_grants;
#[path = "suite/dojo_presets.rs"]
mod dojo_presets;
#[path = "suite/dojo_timeline.rs"]
mod dojo_timeline;
#[path = "suite/durability.rs"]
mod durability;
#[path = "suite/effect_spec.rs"]
mod effect_spec;
#[path = "suite/end_reason.rs"]
mod end_reason;
#[path = "suite/escape_plan.rs"]
mod escape_plan;
#[path = "suite/execute.rs"]
mod execute;
#[path = "suite/field_intent.rs"]
mod field_intent;
#[path = "suite/field_rows.rs"]
mod field_rows;
#[path = "suite/foe_card.rs"]
mod foe_card;
#[path = "suite/forced_move.rs"]
mod forced_move;
#[path = "suite/forecast_move.rs"]
mod forecast_move;
#[path = "suite/gap_close_row.rs"]
mod gap_close_row;
#[path = "suite/gear_consumable_hooks.rs"]
mod gear_consumable_hooks;
#[path = "suite/generated_decor.rs"]
mod generated_decor;
#[path = "suite/ground_hazards.rs"]
mod ground_hazards;
#[path = "suite/guard_timing.rs"]
mod guard_timing;
#[path = "suite/heal_pct.rs"]
mod heal_pct;
#[path = "suite/heal_preview.rs"]
mod heal_preview;
#[path = "suite/hit_fail.rs"]
mod hit_fail;
#[path = "suite/hit_forecast.rs"]
mod hit_forecast;
#[path = "suite/hold_ai.rs"]
mod hold_ai;
#[path = "suite/item_ap.rs"]
mod item_ap;
#[path = "suite/item_pipeline.rs"]
mod item_pipeline;
#[path = "suite/know_potency.rs"]
mod know_potency;
#[path = "suite/log_groups.rs"]
mod log_groups;
#[path = "suite/log_words.rs"]
mod log_words;
#[path = "suite/maneuver.rs"]
mod maneuver;
#[path = "suite/meters.rs"]
mod meters;
#[path = "suite/offhand.rs"]
mod offhand;
#[path = "suite/phase_break.rs"]
mod phase_break;
#[path = "suite/phase_kill.rs"]
mod phase_kill;
#[path = "suite/phase_side_swap.rs"]
mod phase_side_swap;
#[path = "suite/picks.rs"]
mod picks;
#[path = "suite/pk1_hooks.rs"]
mod pk1_hooks;
#[path = "suite/pk1_kitfix.rs"]
mod pk1_kitfix;
#[path = "suite/plan_badges.rs"]
mod plan_badges;
#[path = "suite/plan_recurse.rs"]
mod plan_recurse;
#[path = "suite/poise.rs"]
mod poise;
#[path = "suite/poise_pips.rs"]
mod poise_pips;
#[path = "suite/portrait.rs"]
mod portrait;
#[path = "suite/reach.rs"]
mod reach;
#[path = "suite/relic_hooks.rs"]
mod relic_hooks;
#[path = "suite/retreat.rs"]
mod retreat;
#[path = "suite/riders.rs"]
mod riders;
#[path = "suite/roll_table.rs"]
mod roll_table;
#[path = "suite/round_intent.rs"]
mod round_intent;
#[path = "suite/round_intent_plan.rs"]
mod round_intent_plan;
#[path = "suite/ruleset_switches.rs"]
mod ruleset_switches;
#[path = "suite/script_round.rs"]
mod script_round;
#[path = "suite/shape_direction.rs"]
mod shape_direction;
#[path = "suite/skill_reach.rs"]
mod skill_reach;
#[path = "suite/skill_unmark_turned.rs"]
mod skill_unmark_turned;
#[path = "suite/stagger_trace.rs"]
mod stagger_trace;
#[path = "suite/start_ap.rs"]
mod start_ap;
#[path = "suite/status_numbers.rs"]
mod status_numbers;
#[path = "suite/status_view.rs"]
mod status_view;
#[path = "suite/stay_behind.rs"]
mod stay_behind;
#[path = "suite/tactics.rs"]
mod tactics;
#[path = "suite/techniques.rs"]
mod techniques;
#[path = "suite/terrain_rows.rs"]
mod terrain_rows;
#[path = "suite/tier_ai_cap.rs"]
mod tier_ai_cap;
#[path = "suite/timeline_timing.rs"]
mod timeline_timing;
#[path = "suite/top_bar_lines.rs"]
mod top_bar_lines;
#[path = "suite/triggers.rs"]
mod triggers;
#[path = "suite/unit_tracks.rs"]
mod unit_tracks;
#[path = "suite/weapon_instances.rs"]
mod weapon_instances;
