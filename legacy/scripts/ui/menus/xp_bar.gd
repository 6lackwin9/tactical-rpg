class_name XpBar
extends Control

# ==============================================================================
# Tactical RPG - XP bar (BATTLE-FLOW): progress to the next level. `fill` (0..1) is the filled share of the current
# level; `gain` (0..fill) is the part of it earned this battle, drawn highlighted at the end of the fill.
# ==============================================================================

const COL_BACK: Color = Color(0.08, 0.10, 0.14, 1.0)
const COL_FILL: Color = Color(0.25, 0.45, 0.70, 1.0)
const COL_GAIN: Color = Color(0.95, 0.80, 0.30, 1.0)
const COL_EDGE: Color = Color(0.45, 0.6, 0.8, 1.0)
const HEIGHT: float = 16.0

var fill: float = 0.0
var gain: float = 0.0

func _init(p_fill: float = 0.0, p_gain: float = 0.0) -> void:
	fill = clampf(p_fill, 0.0, 1.0)
	gain = clampf(p_gain, 0.0, fill)
	custom_minimum_size = Vector2(120, HEIGHT)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_SHRINK_CENTER

func _draw() -> void:
	var w: float = size.x
	var h: float = size.y
	draw_rect(Rect2(0, 0, w, h), COL_BACK)
	var old_w: float = (fill - gain) * w
	draw_rect(Rect2(0, 0, old_w, h), COL_FILL)
	draw_rect(Rect2(old_w, 0, gain * w, h), COL_GAIN)
	draw_rect(Rect2(0, 0, w, h), COL_EDGE, false, 2.0)
