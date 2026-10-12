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

func refresh() -> void:
	if session == null:
		return
	dock.update_view(session.timeline_unit(), session.playhead(), session.is_timeline_masked(), session.can_view_prev_round(), session.can_view_next_round())
