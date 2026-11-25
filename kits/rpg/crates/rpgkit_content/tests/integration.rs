//! Consolidated integration test harness for `rpgkit_content`.
//!
//! Groups 100 integration suites under one binary to avoid redundant link steps.
//! All integration suites run in this single binary.

#[path = "suite/area_affects_field.rs"]
mod area_affects_field;
#[path = "suite/battle_hook_checks.rs"]
mod battle_hook_checks;
#[path = "suite/battle_script_checks.rs"]
mod battle_script_checks;
#[path = "suite/battle_talk_checks.rs"]
mod battle_talk_checks;
#[path = "suite/big_unit_checks.rs"]
mod big_unit_checks;
#[path = "suite/body_slot_checks.rs"]
mod body_slot_checks;
#[path = "suite/campaign_hook_checks.rs"]
mod campaign_hook_checks;
#[path = "suite/campaign_schema.rs"]
mod campaign_schema;
#[path = "suite/channel_checks.rs"]
mod channel_checks;
#[path = "suite/class_basics_checks.rs"]
mod class_basics_checks;
#[path = "suite/class_keystone_checks.rs"]
mod class_keystone_checks;
#[path = "suite/class_passives_checks.rs"]
mod class_passives_checks;
#[path = "suite/class_shares_checks.rs"]
mod class_shares_checks;
#[path = "suite/decor.rs"]
mod decor;
#[path = "suite/derived_stats.rs"]
mod derived_stats;
#[path = "suite/dialogue_conditions.rs"]
mod dialogue_conditions;
#[path = "suite/dialogue_roll.rs"]
mod dialogue_roll;
#[path = "suite/dojo_preset_checks.rs"]
mod dojo_preset_checks;
#[path = "suite/duplicate_skills.rs"]
mod duplicate_skills;
#[path = "suite/empower_grant.rs"]
mod empower_grant;
#[path = "suite/encounter_rows.rs"]
mod encounter_rows;
#[path = "suite/ending_checks.rs"]
mod ending_checks;
#[path = "suite/enrage_checks.rs"]
mod enrage_checks;
#[path = "suite/faces.rs"]
mod faces;
#[path = "suite/facility_checks.rs"]
mod facility_checks;
#[path = "suite/family_mastery.rs"]
mod family_mastery;
#[path = "suite/family_range_band.rs"]
mod family_range_band;
#[path = "suite/family_rules.rs"]
mod family_rules;
#[path = "suite/flag_skill_checks.rs"]
mod flag_skill_checks;
#[path = "suite/fx_preset_checks.rs"]
mod fx_preset_checks;
#[path = "suite/gear_consumable_hook_checks.rs"]
mod gear_consumable_hook_checks;
#[path = "suite/gear_rework_checks.rs"]
mod gear_rework_checks;
#[path = "suite/ground_checks.rs"]
mod ground_checks;
#[path = "suite/growth_per_stage.rs"]
mod growth_per_stage;
#[path = "suite/guard_rows.rs"]
mod guard_rows;
#[path = "suite/hand_rule_checks.rs"]
mod hand_rule_checks;
#[path = "suite/heal_pct_checks.rs"]
mod heal_pct_checks;
#[path = "suite/hooks_schema.rs"]
mod hooks_schema;
#[path = "suite/install_effect_checks.rs"]
mod install_effect_checks;
#[path = "suite/item_area_field.rs"]
mod item_area_field;
#[path = "suite/look1_sets.rs"]
mod look1_sets;
#[path = "suite/look_fields.rs"]
mod look_fields;
#[path = "suite/marker_face.rs"]
mod marker_face;
#[path = "suite/mastery_checks.rs"]
mod mastery_checks;
#[path = "suite/phase_form_checks.rs"]
mod phase_form_checks;
#[path = "suite/phase_kill_checks.rs"]
mod phase_kill_checks;
#[path = "suite/phase_side_swap_checks.rs"]
mod phase_side_swap_checks;
#[path = "suite/pk1_hook_checks.rs"]
mod pk1_hook_checks;
#[path = "suite/pk1_kitfix_checks.rs"]
mod pk1_kitfix_checks;
#[path = "suite/plate_names.rs"]
mod plate_names;
#[path = "suite/point_use.rs"]
mod point_use;
#[path = "suite/portrait_cues.rs"]
mod portrait_cues;
#[path = "suite/portrait_expressions.rs"]
mod portrait_expressions;
#[path = "suite/portrait_key.rs"]
mod portrait_key;
#[path = "suite/portrait_overlay_switch.rs"]
mod portrait_overlay_switch;
#[path = "suite/purse_caps.rs"]
mod purse_caps;
#[path = "suite/rank_modifier_keys.rs"]
mod rank_modifier_keys;
#[path = "suite/reach_rule.rs"]
mod reach_rule;
#[path = "suite/recipe_checks.rs"]
mod recipe_checks;
#[path = "suite/relic_hook_checks.rs"]
mod relic_hook_checks;
#[path = "suite/retired_block_cost.rs"]
mod retired_block_cost;
#[path = "suite/retired_buff_stat.rs"]
mod retired_buff_stat;
#[path = "suite/revive_checks.rs"]
mod revive_checks;
#[path = "suite/roll_table_checks.rs"]
mod roll_table_checks;
#[path = "suite/roof_checks.rs"]
mod roof_checks;
#[path = "suite/roster_schema.rs"]
mod roster_schema;
#[path = "suite/rule_effect_gate_checks.rs"]
mod rule_effect_gate_checks;
#[path = "suite/ruleset_modes.rs"]
mod ruleset_modes;
#[path = "suite/rulesets.rs"]
mod rulesets;
#[path = "suite/shake_schema.rs"]
mod shake_schema;
#[path = "suite/shop_discount_checks.rs"]
mod shop_discount_checks;
#[path = "suite/siege_checks.rs"]
mod siege_checks;
#[path = "suite/skill_family_defaults.rs"]
mod skill_family_defaults;
#[path = "suite/skill_fold_error.rs"]
mod skill_fold_error;
#[path = "suite/skill_reach_checks.rs"]
mod skill_reach_checks;
#[path = "suite/skill_unmark_checks.rs"]
mod skill_unmark_checks;
#[path = "suite/stance_tokens.rs"]
mod stance_tokens;
#[path = "suite/stay_behind_checks.rs"]
mod stay_behind_checks;
#[path = "suite/story_checks.rs"]
mod story_checks;
#[path = "suite/supplies_checks.rs"]
mod supplies_checks;
#[path = "suite/survivors_checks.rs"]
mod survivors_checks;
#[path = "suite/talk_checks.rs"]
mod talk_checks;
#[path = "suite/temperament.rs"]
mod temperament;
#[path = "suite/terms_checks.rs"]
mod terms_checks;
#[path = "suite/terrain_fields.rs"]
mod terrain_fields;
#[path = "suite/terrain_partial.rs"]
mod terrain_partial;
#[path = "suite/tier_checks.rs"]
mod tier_checks;
#[path = "suite/timeline_fields.rs"]
mod timeline_fields;
#[path = "suite/unit_drills.rs"]
mod unit_drills;
#[path = "suite/unit_pool_checks.rs"]
mod unit_pool_checks;
#[path = "suite/unit_template_checks.rs"]
mod unit_template_checks;
#[path = "suite/unit_track_checks.rs"]
mod unit_track_checks;
#[path = "suite/until_bunk_checks.rs"]
mod until_bunk_checks;
#[path = "suite/weapon_durability.rs"]
mod weapon_durability;
#[path = "suite/weapon_types.rs"]
mod weapon_types;
#[path = "suite/world_manifest.rs"]
mod world_manifest;
#[path = "suite/world_map_fields.rs"]
mod world_map_fields;
#[path = "suite/world_map_layout.rs"]
mod world_map_layout;
#[path = "suite/world_map_options.rs"]
mod world_map_options;
#[path = "suite/zones.rs"]
mod zones;
