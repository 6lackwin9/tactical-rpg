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

func refresh() -> void:
	if session == null or session.grid == null:
		return
	var c: Combatant = session.selected_combatant
	if c == null or not is_instance_valid(c):
		return
	var target: Combatant = session.forecast_target(c)
	if target == null:
		return
	forecast.update_forecast(c, target, session.grid)
