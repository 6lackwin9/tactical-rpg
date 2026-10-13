class_name TacticalForecastUI
extends PanelContainer

# ==============================================================================
# Tactical RPG - Tactical Forecast & Arena Intel Dock
# Right-side tactical dock displaying target distance, range verification,
# stance matchup multipliers, estimated damage, and arena landmark intel.
# ==============================================================================

var forecast_target_lbl: Label
var forecast_dist_lbl: Label
var forecast_range_badge: Label
var forecast_stance_lbl: Label
var forecast_dmg_lbl: Label
var intel_center_lbl: Label
var intel_exit_lbl: Label
var intel_turn_lbl: Label

func _init() -> void:
	name = "TacticalForecastUI"
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	custom_minimum_size = Vector2(340, 468)

	var right_style := StyleBoxFlat.new()
	right_style.bg_color = Color(0.08, 0.10, 0.15, 0.95)
	right_style.border_color = Color(0.2, 0.28, 0.42, 0.8)
	right_style.set_border_width_all(1)
	right_style.set_corner_radius_all(6)
	right_style.content_margin_left = 8
	right_style.content_margin_right = 8
	right_style.content_margin_top = 6
	right_style.content_margin_bottom = 6
	add_theme_stylebox_override("panel", right_style)

	_build_ui()

func _build_ui() -> void:
	var right_vbox := VBoxContainer.new()
	right_vbox.add_theme_constant_override("separation", 6)
	add_child(right_vbox)

	var forecast_hdr := Label.new()
	forecast_hdr.text = "TACTICAL FORECAST & ARENA INTEL"
	forecast_hdr.add_theme_font_size_override("font_size", 11)
	forecast_hdr.add_theme_color_override("font_color", Color(0.65, 0.8, 1.0))
	right_vbox.add_child(forecast_hdr)

	# Tactical Forecast Box
	var forecast_box := PanelContainer.new()
	var fc_style := StyleBoxFlat.new()
	fc_style.bg_color = Color(0.12, 0.15, 0.22, 0.8)
	fc_style.border_color = Color(0.3, 0.4, 0.55, 0.6)
	fc_style.set_border_width_all(1)
	fc_style.set_corner_radius_all(6)
	fc_style.content_margin_left = 8
	fc_style.content_margin_right = 8
	fc_style.content_margin_top = 6
	fc_style.content_margin_bottom = 6
	forecast_box.add_theme_stylebox_override("panel", fc_style)
	right_vbox.add_child(forecast_box)

	var fc_vbox := VBoxContainer.new()
	fc_vbox.add_theme_constant_override("separation", 4)
	forecast_box.add_child(fc_vbox)

	forecast_target_lbl = Label.new()
	forecast_target_lbl.text = "Target: Enemy Brawler"
	forecast_target_lbl.add_theme_font_size_override("font_size", 11)
	forecast_target_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.4))
	fc_vbox.add_child(forecast_target_lbl)

	forecast_dist_lbl = Label.new()
	forecast_dist_lbl.text = "Distance: 1 Tile"
	forecast_dist_lbl.add_theme_font_size_override("font_size", 10)
	forecast_dist_lbl.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	fc_vbox.add_child(forecast_dist_lbl)

	forecast_range_badge = Label.new()
	forecast_range_badge.text = "✓ IN ATTACK RANGE"
	forecast_range_badge.add_theme_font_size_override("font_size", 10)
	forecast_range_badge.add_theme_color_override("font_color", Color(0.4, 0.9, 0.4))
	fc_vbox.add_child(forecast_range_badge)

	forecast_stance_lbl = Label.new()
	forecast_stance_lbl.text = "Stance Matchup: Neutral vs Defensive"
	forecast_stance_lbl.add_theme_font_size_override("font_size", 10)
	forecast_stance_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
	fc_vbox.add_child(forecast_stance_lbl)

	forecast_dmg_lbl = Label.new()
	forecast_dmg_lbl.text = "Estimated Strike: ~12 dmg (Defending)"
	forecast_dmg_lbl.add_theme_font_size_override("font_size", 10)
	forecast_dmg_lbl.add_theme_color_override("font_color", Color(0.95, 0.5, 0.5))
	fc_vbox.add_child(forecast_dmg_lbl)

	# Arena Rules / Objectives
	var arena_hdr := Label.new()
	arena_hdr.text = "ARENA RULES & OBJECTIVES"
	arena_hdr.add_theme_font_size_override("font_size", 10)
	arena_hdr.add_theme_color_override("font_color", Color(0.85, 0.8, 0.55))
	right_vbox.add_child(arena_hdr)

	var arena_card := PanelContainer.new()
	var ar_style := StyleBoxFlat.new()
	ar_style.bg_color = Color(0.10, 0.13, 0.18, 0.8)
	ar_style.border_color = Color(0.22, 0.3, 0.42, 0.6)
	ar_style.set_border_width_all(1)
	ar_style.set_corner_radius_all(6)
	ar_style.content_margin_left = 8
	ar_style.content_margin_right = 8
	ar_style.content_margin_top = 6
	ar_style.content_margin_bottom = 6
	arena_card.add_theme_stylebox_override("panel", ar_style)
	right_vbox.add_child(arena_card)

	var ar_vbox := VBoxContainer.new()
	ar_vbox.add_theme_constant_override("separation", 3)
	arena_card.add_child(ar_vbox)

	intel_center_lbl = Label.new()
	intel_center_lbl.text = "🚩 Center Ring (4, 4): Core arena landmark"
	intel_center_lbl.add_theme_font_size_override("font_size", 9)
	intel_center_lbl.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	ar_vbox.add_child(intel_center_lbl)

	intel_exit_lbl = Label.new()
	intel_exit_lbl.text = "🚪 Exit Tile (8, 4): Allows safe withdrawal"
	intel_exit_lbl.add_theme_font_size_override("font_size", 9)
	intel_exit_lbl.add_theme_color_override("font_color", Color(0.9, 0.8, 0.4))
	ar_vbox.add_child(intel_exit_lbl)

	intel_turn_lbl = Label.new()
	intel_turn_lbl.text = "⏱ Blocks / Round"
	intel_turn_lbl.add_theme_font_size_override("font_size", 9)
	intel_turn_lbl.add_theme_color_override("font_color", Color(0.6, 0.8, 1.0))
	ar_vbox.add_child(intel_turn_lbl)

	var right_hint := Label.new()
	right_hint.text = "💡 Click any enemy unit to inspect their threat level."
	right_hint.add_theme_font_size_override("font_size", 8)
	right_hint.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	right_vbox.add_child(right_hint)

# Arena rules line: blocks per round, AP pool and recovery, exits.
func set_arena_intel(total_blocks: int, unit_ap: int, ap_regen: int, exits: Array[Vector2i]) -> void:
	intel_turn_lbl.text = "⏱ %d Blocks / Round • %d AP pool, %s each round" % [total_blocks, unit_ap, ("+%d" % ap_regen) if ap_regen >= 0 else "refilled"]
	var exit_names: Array[String] = []
	for e: Vector2i in exits:
		exit_names.append(str(e))
	intel_exit_lbl.text = ("🚪 Exit %s: withdraw to leave the battle" % ", ".join(exit_names)) if not exit_names.is_empty() else "🚪 No exit: fight to the end"

# `power`: the unit's strike power; `band`: its basic attack's reach (min, max).
func update_forecast(selected_combatant: Combatant, target: Combatant, grid: CombatGrid, power: int = 25, band: Vector2i = Vector2i(1, 1)) -> void:
	if not selected_combatant:
		return

	if target == null:
		forecast_target_lbl.text = "Target: None"
		forecast_dist_lbl.text = "Distance: N/A"
		forecast_range_badge.text = "Status: No Target"
		forecast_range_badge.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
		forecast_stance_lbl.text = "Stance: N/A"
		forecast_dmg_lbl.text = "Est. Strike: N/A"
		return

	forecast_target_lbl.text = "Target: %s" % target.name
	if selected_combatant.is_active() and target.is_active():
		var p1: Vector2i = grid.get_position(selected_combatant)
		var p2: Vector2i = grid.get_position(target)
		var dist: int = grid.get_distance(p1, p2)
		forecast_dist_lbl.text = "Distance: %d tile%s (Manhattan)" % [dist, "s" if dist != 1 else ""]

		if dist >= band.x and dist <= band.y:
			forecast_range_badge.text = "✓ IN REACH (distance %d, reach %d-%d)" % [dist, band.x, band.y]
			forecast_range_badge.add_theme_color_override("font_color", Color(0.4, 0.9, 0.4))
		elif dist < band.x:
			forecast_range_badge.text = "✗ TOO CLOSE (reach %d-%d): back off or off-hand jab" % [band.x, band.y]
			forecast_range_badge.add_theme_color_override("font_color", Color(0.98, 0.7, 0.3))
		else:
			forecast_range_badge.text = "✗ OUT OF REACH (+%d tiles): Engage moves in" % (dist - band.y)
			forecast_range_badge.add_theme_color_override("font_color", Color(0.95, 0.4, 0.4))

		var mult: float = CombatTypes.get_matchup_multiplier(selected_combatant.current_stance, target.current_stance)
		forecast_stance_lbl.text = "Stance: %s vs %s (%.2fx)" % [
			CombatTypes.stance_to_string(selected_combatant.current_stance),
			CombatTypes.stance_to_string(target.current_stance),
			mult
		]

		var est_dmg: int = int(roundf(float(power) * mult * selected_combatant.attack_boost_multiplier * (1.0 - target.defense_mitigation_ratio)))
		var mit_note: String = " (-%d%% def)" % int(target.defense_mitigation_ratio * 100) if target.defense_mitigation_ratio > 0 else ""
		forecast_dmg_lbl.text = "Est. Strike Impact: ~%d dmg%s" % [est_dmg, mit_note]
	else:
		forecast_dist_lbl.text = "Distance: Inactive combatant"
		forecast_range_badge.text = "Status: Off Active Grid"
		forecast_range_badge.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
		forecast_stance_lbl.text = "Stance: N/A"
		forecast_dmg_lbl.text = "Est. Strike: N/A"
