class_name CombatForecastPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Tactical Forecast Panel
# Range check, stance matchup and estimated damage between the selected unit
# and its locked target (or the nearest opposing unit), plus arena intel.
# ==============================================================================

var forecast: TacticalForecastUI

func _init() -> void:
	super._init()
	panel_id = "forecast"
	panel_title = "Tactical Forecast"
	panel_icon = "🎯"
	default_min_size = Vector2(340, 468)
	forecast = TacticalForecastUI.new()
	add_child(forecast)

func _on_bound() -> void:
	session.hover_changed.connect(func(_c: Combatant) -> void: refresh())

func refresh() -> void:
	if session == null or session.grid == null:
		return
	var c: Combatant = session.selected_combatant
	var hovered_foe: Combatant = session.hovered_combatant if session.is_planning() and session.hovered_combatant != null and not session.is_ally(session.hovered_combatant) else null
	if hovered_foe != null and session.focused_hero != null:
		c = session.focused_hero   # planning: forecast the hero against the hovered enemy
	if c == null or not is_instance_valid(c):
		return
	var target: Combatant = hovered_foe if hovered_foe != null else session.forecast_target(c)
	if target == null:
		return
	forecast.update_forecast(c, target, session.grid)
