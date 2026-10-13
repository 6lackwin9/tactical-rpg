class_name PlanOverlay
extends Control

# ==============================================================================
# Tactical RPG - Plan Overlay (planning ghosts drawn on the combat grid)
# Shows, for the hero being planned: the route its committed sequence walks
# (numbered), a ghost token where the sequence leaves it, and - while hovering
# an option - the previewed route, destination ghost, weapon reach and target
# ring plus a short cost hint. Purely visual; mouse input passes through.
# ==============================================================================

const TILE: int = 54
const COL_COMMITTED: Color = Color(1.0, 0.85, 0.3, 0.9)
const COL_PREVIEW: Color = Color(0.35, 0.9, 1.0, 0.95)
const COL_REACH: Color = Color(1.0, 0.35, 0.35, 0.16)
const NONE: Vector2i = Vector2i(-1, -1)

var ghost_texture: Texture2D = null
var committed_trail: Array[Vector2i] = []    # every tile walked, from the start tile
var committed_marks: Array[Vector2i] = []    # tiles where a move block ends (numbered)
var preview_marks: Array[Vector2i] = []
var committed_ghost: Vector2i = NONE         # where the committed sequence leaves the hero
var preview_trail: Array[Vector2i] = []
var preview_ghost: Vector2i = NONE
var preview_target: Vector2i = NONE
var reach_tiles: Array[Vector2i] = []
var hint_text: String = ""
var hint_cell: Vector2i = NONE

var _hint_label: Label

func _init() -> void:
	name = "PlanOverlay"
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_hint_label = Label.new()
	_hint_label.add_theme_font_size_override("font_size", 9)
	_hint_label.add_theme_color_override("font_color", Color(0.9, 1.0, 1.0))
	_hint_label.add_theme_color_override("font_outline_color", Color(0.02, 0.05, 0.1, 0.95))
	_hint_label.add_theme_constant_override("outline_size", 4)
	_hint_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_hint_label.z_index = 40
	_hint_label.visible = false
	add_child(_hint_label)

func clear() -> void:
	committed_trail = []
	committed_marks = []
	preview_marks = []
	committed_ghost = NONE
	preview_trail = []
	preview_ghost = NONE
	preview_target = NONE
	reach_tiles = []
	hint_text = ""
	hint_cell = NONE
	_hint_label.visible = false
	queue_redraw()

func refresh_hint() -> void:
	_hint_label.visible = not hint_text.is_empty() and hint_cell != NONE
	if _hint_label.visible:
		_hint_label.text = hint_text
		var pos: Vector2 = Vector2(hint_cell) * TILE + Vector2(TILE + 2, -2)
		if pos.x > size.x - 150.0:
			pos.x = Vector2(hint_cell).x * TILE - 150.0
		_hint_label.position = Vector2(maxf(0.0, pos.x), maxf(0.0, pos.y))
	queue_redraw()

static func _center(cell: Vector2i) -> Vector2:
	return Vector2(cell) * TILE + Vector2(TILE, TILE) * 0.5

func _draw() -> void:
	for cell: Vector2i in reach_tiles:
		draw_rect(Rect2(Vector2(cell) * TILE + Vector2(2, 2), Vector2(TILE - 4, TILE - 4)), COL_REACH)
	_draw_trail(committed_trail, committed_marks, COL_COMMITTED, true)
	_draw_ghost(committed_ghost, 0.45)
	_draw_trail(preview_trail, preview_marks, COL_PREVIEW, false)
	_draw_ghost(preview_ghost, 0.7)
	if preview_target != NONE:
		draw_arc(_center(preview_target), TILE * 0.46, 0.0, TAU, 28, Color(1.0, 0.35, 0.35, 0.95), 2.5)

func _draw_trail(trail: Array[Vector2i], marks: Array[Vector2i], col: Color, numbered: bool) -> void:
	if trail.size() < 2:
		return
	for i: int in range(trail.size() - 1):
		draw_line(_center(trail[i]), _center(trail[i + 1]), col, 3.0)
	for i: int in range(marks.size()):
		var c: Vector2 = _center(marks[i])
		draw_circle(c, 7.0, col)
		if numbered:
			draw_string(ThemeDB.fallback_font, c + Vector2(-3, 4), str(i + 1), HORIZONTAL_ALIGNMENT_LEFT, -1, 9, Color(0.1, 0.1, 0.1, 1.0))
func _draw_ghost(cell: Vector2i, alpha: float) -> void:
	if cell == NONE:
		return
	var rect := Rect2(Vector2(cell) * TILE + Vector2(5, 5), Vector2(TILE - 10, TILE - 10))
	if ghost_texture != null:
		draw_texture_rect(ghost_texture, rect, false, Color(1, 1, 1, alpha))
	else:
		draw_rect(rect, Color(1, 1, 1, alpha * 0.5))
	draw_rect(rect, Color(1, 1, 1, alpha), false, 1.5)