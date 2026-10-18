---
class: TacticalForecastUI
extends: PanelContainer
description: "Tactical RPG - Tactical Forecast & Arena Intel Dock Right-side tactical dock displaying target distance, range verification, stance matchup multipliers, estimated damage, and arena landmark intel."
---

# Properties
* `PANEL_WIDTH`: float = 220.0 (const)
* `forecast_target_lbl`: Label
* `forecast_dist_lbl`: Label
* `forecast_range_badge`: Label
* `forecast_stance_lbl`: Label
* `forecast_dmg_lbl`: Label
* `intel_turn_lbl`: Label
* `forecast_threat_lbl`: Label

# Methods

### `_init() -> void`
```logic
name = "TacticalForecastUI"
size_flags_horizontal = Control.SIZE_EXPAND_FILL
size_flags_vertical = Control.SIZE_EXPAND_FILL
custom_minimum_size = Vector2(0, 0)
clip_contents = true   # the width is fixed: long text wraps (a scroll takes over when it is too tall), it never widens the panel
add_theme_stylebox_override("panel", UiTheme.box(Color(0.08, 0.10, 0.15, 0.95), Color(0.2, 0.28, 0.42, 0.8), 6, 6, CombatHudMetrics.PANEL_PAD))
_build_ui()
```

### `_build_ui() -> void`
```logic
var scroll := ScrollContainer.new()
scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
add_child(scroll)
var box := VBoxContainer.new()
box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
box.add_theme_constant_override("separation", CombatHudMetrics.CARD_GAP)
scroll.add_child(box)
var fs: int = UiTheme.hud_font()
box.add_child(_line("FORECAST", fs, Color(0.65, 0.8, 1.0)))
forecast_target_lbl = _line("Target: none", fs, Color(1.0, 0.85, 0.4))
forecast_dist_lbl = _line("Distance: -", fs, Color(0.8, 0.9, 1.0))
forecast_range_badge = _line("", fs, Color(0.4, 0.9, 0.4))
forecast_stance_lbl = _line("Stance: -", fs, Color(0.7, 0.8, 0.9))
forecast_dmg_lbl = _line("Strike: -", fs, Color(0.95, 0.5, 0.5))
intel_turn_lbl = _line("", fs, Color(0.6, 0.8, 1.0))   # the one arena rules line
forecast_threat_lbl = _line("", fs, Color(1.0, 0.5, 0.45))
forecast_threat_lbl.visible = false
for lbl: Label in [forecast_target_lbl, forecast_dist_lbl, forecast_range_badge, forecast_stance_lbl, forecast_dmg_lbl, forecast_threat_lbl, intel_turn_lbl]:
	box.add_child(lbl)
```

### `_line(text: String, fs: int, color: Color) -> Label`
```logic
var lbl: Label = UiTheme.label(text, fs, color)
lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
return lbl
```

### `set_threat(text: String) -> void`
```logic
forecast_threat_lbl.text = text
forecast_threat_lbl.visible = not text.is_empty()
```

### `set_arena_intel(total_blocks: int, unit_ap: int, ap_regen: int, exits: Array[Vector2i]) -> void`
```logic
var regen: String = ("+%d" % ap_regen) if ap_regen >= 0 else "refill"
intel_turn_lbl.text = "%d blocks - %d AP %s - %s" % [total_blocks, unit_ap, regen, ("exit " + str(exits[0])) if not exits.is_empty() else "no exit"]
```

### `update_forecast(selected_combatant: Combatant, target: Combatant, grid: CombatGrid, expected_hit: int = 0, band: Vector2i = Vector2i(1, 1)) -> void`
```logic
if not selected_combatant:
	return

if target == null:
	forecast_target_lbl.text = "Target: none"
	forecast_dist_lbl.text = "Distance: -"
	forecast_range_badge.text = "No target"
	forecast_range_badge.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
	forecast_stance_lbl.text = "Stance: -"
	forecast_dmg_lbl.text = "Strike: -"
	return

forecast_target_lbl.text = "Target: %s" % target.name
if selected_combatant.is_active() and target.is_active():
	var p1: Vector2i = grid.get_position(selected_combatant)
	var p2: Vector2i = grid.get_position(target)
	var dist: int = grid.get_distance(p1, p2)
	forecast_dist_lbl.text = "Distance %d, reach %d-%d" % [dist, band.x, band.y]

	if dist >= band.x and dist <= band.y:
		forecast_range_badge.text = "In reach"
		forecast_range_badge.add_theme_color_override("font_color", Color(0.4, 0.9, 0.4))
	elif dist < band.x:
		forecast_range_badge.text = "Too close: back off"
		forecast_range_badge.add_theme_color_override("font_color", Color(0.98, 0.7, 0.3))
	else:
		forecast_range_badge.text = "Out of reach (+%d tiles)" % (dist - band.y)
		forecast_range_badge.add_theme_color_override("font_color", Color(0.95, 0.4, 0.4))

	var mult: float = CombatTypes.get_matchup_multiplier(selected_combatant.current_stance, target.current_stance)
	forecast_stance_lbl.text = "Stance %s vs %s %.2fx" % [
		CombatTypes.stance_to_string(selected_combatant.current_stance),
		CombatTypes.stance_to_string(target.current_stance),
		mult
	]

	var est_dmg: int = maxi(1, roundi(float(expected_hit) * (1.0 - target.defense_mitigation_ratio)))
	var mit_note: String = " (-%d%% guard)" % roundi(target.defense_mitigation_ratio * 100.0) if target.defense_mitigation_ratio > 0 else ""
	forecast_dmg_lbl.text = "Strike ~%d%s, crit %.0f%%" % [est_dmg, mit_note, DamageCalculator.crit_chance(selected_combatant)]
else:
	forecast_dist_lbl.text = "Distance: -"
	forecast_range_badge.text = "Off the grid"
	forecast_range_badge.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
	forecast_stance_lbl.text = "Stance: -"
	forecast_dmg_lbl.text = "Strike: -"
```

