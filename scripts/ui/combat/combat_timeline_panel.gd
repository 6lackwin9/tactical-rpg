class_name CombatTimelinePanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Timeline Dock Panel
# 10-block ruler plus the focused unit's action track for the viewed round.
# Enemy tracks stay masked during the live round. ◀ ▶ traverse rounds; clicking
# a block inspects that block's state.
# ==============================================================================

var dock: TimelineDockUI

func _init() -> void:
	super._init()
	panel_id = "timeline"
	panel_title = "Timeline"
	panel_icon = "⏱"
	default_min_size = Vector2(600, 90)
	custom_minimum_size = Vector2(0, 88)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	dock = TimelineDockUI.new()
	add_child(dock)

func _on_bound() -> void:
	dock.action_lookup_fn = session.find_recorded_action
	dock.round_nav_requested.connect(func(delta: int) -> void: session.navigate_round(delta))
	dock.block_selected.connect(func(b: int) -> void: session.select_block(b))
	dock.combatant_selected.connect(func(c: Combatant) -> void: session.select_combatant(c))
	dock.hero_view_requested.connect(func() -> void: session.focus_first_hero())
	dock.draft_requested.connect(func(preset_id: String, whole_party: bool) -> void:
		if whole_party:
			session.plan_apply_preset_all(preset_id)
		else:
			session.plan_apply_preset(session.focused_hero, preset_id)
	)
	dock.clear_requested.connect(func() -> void: session.plan_clear(session.focused_hero))
	dock.confirm_requested.connect(func() -> void: session.confirm_plan())
	dock.step_remove_requested.connect(func(i: int) -> void: session.plan_remove_step(session.focused_hero, i))
	dock.step_reorder_requested.connect(func(from_i: int, to_i: int) -> void: session.plan_move_step(session.focused_hero, from_i, to_i))

func refresh() -> void:
	if session == null:
		return
	dock.set_planning(session.plan_info(), session.plan_step_index_at)
	dock.update_view(session.timeline_unit(), session.playhead(), session.is_timeline_masked(), session.can_view_prev_round(), session.can_view_next_round())
