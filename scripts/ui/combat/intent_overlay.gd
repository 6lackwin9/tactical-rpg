class_name IntentOverlay
extends Control

# ==============================================================================
# Tactical RPG - Intent overlay (INTENT1-U)
# Planning only: a small kind chip on every enemy token (ui key `intent_<kind>`) and a hostile-coloured arrow from the
# enemy to its target token, or to its aim cell for area and move (dashed for move). It sits above the tokens and below
# the unit hover card (which is drawn later with a higher z_index), so it never covers the card. Display only; mouse
# input passes through. CombatGridPanel feeds it from CombatSession.enemy_intents() and clears it outside planning.
# ==============================================================================

const TILE: int = 54
const COL_ARROW: Color = Color(1.0, 0.35, 0.35, 0.9)
const CHIP_MIN: int = 18
const CHIP_MAX: int = 26

var tile: int = TILE
var arrows: Array[Dictionary] = []    # {from: Vector2i, to: Vector2i, dashed: bool}
var chips: Dictionary = {}            # Combatant -> TextureRect
var _cells: Dictionary = {}           # Combatant -> Vector2i (where its chip sits)

func _init() -> void:
	name = "IntentOverlay"
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	z_index = 1   # above the tokens; the hover card is z_index 50

func chip_px() -> int:
	return clampi(roundi(float(tile) * 0.42), CHIP_MIN, CHIP_MAX)

# Shows the tells of `records` (IntentRules records); units' positions come from `grid`.
func show_tells(records: Array[Dictionary], grid: CombatGrid) -> void:
	arrows = []
	var keep: Dictionary = {}
	for rec: Dictionary in records:
		var unit: Combatant = rec["unit"]
		if grid == null or not grid.has_combatant(unit):
			continue
		var from: Vector2i = grid.get_position(unit)
		keep[unit] = true
		_cells[unit] = from
		var chip: TextureRect = _chip_for(unit)
		var kind: String = str(rec["kind"])
		chip.texture = AssetRegistry.texture("ui", "intent_" + kind, "icon")
		chip.tooltip_text = IntentText.first_line(rec)
		var target: Variant = rec.get("target", null)
		var aim: Vector2i = rec.get("cell", IntentRules.NO_CELL)
		var to: Vector2i = IntentRules.NO_CELL
		if target is Combatant:
			var foe: Combatant = target
			if grid.has_combatant(foe):
				to = grid.get_position(foe)
		elif aim != IntentRules.NO_CELL:
			to = aim
		if to != IntentRules.NO_CELL and to != from:
			arrows.append({"from": from, "to": to, "dashed": kind == "move" or kind == "flee"})
	for unit_key: Variant in chips.keys():
		if not keep.has(unit_key):
			var stale: TextureRect = chips[unit_key]
			chips.erase(unit_key)
			_cells.erase(unit_key)
			remove_child(stale)
			stale.queue_free()
	_place_chips()
	queue_redraw()

func clear() -> void:
	if arrows.is_empty() and chips.is_empty():
		return
	arrows = []
	for chip_raw: Variant in chips.values():
		var chip: TextureRect = chip_raw
		remove_child(chip)
		chip.queue_free()
	chips.clear()
	_cells.clear()
	queue_redraw()

func set_tile(px: int) -> void:
	tile = px
	_place_chips()
	queue_redraw()

func chip_count() -> int:
	return chips.size()

func arrow_count() -> int:
	return arrows.size()

func _chip_for(unit: Combatant) -> TextureRect:
	var raw: Variant = chips.get(unit, null)
	if raw is TextureRect:
		return raw
	var chip := TextureRect.new()
	chip.name = "Chip_%s" % unit.name
	chip.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	chip.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	chip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(chip)
	chips[unit] = chip
	return chip

# The chip sits in the top-left corner of the enemy's tile.
func _place_chips() -> void:
	var s: float = float(chip_px())
	for unit_key: Variant in chips:
		var chip: TextureRect = chips[unit_key]
		var cell: Vector2i = _cells.get(unit_key, IntentRules.NO_CELL)
		chip.size = Vector2(s, s)
		chip.custom_minimum_size = Vector2(s, s)
		chip.position = Vector2(cell) * float(tile) + Vector2(1, 1)

func _center(cell: Vector2i) -> Vector2:
	return Vector2(cell) * float(tile) + Vector2(tile, tile) * 0.5

func _draw() -> void:
	for arrow: Dictionary in arrows:
		var from_cell: Vector2i = arrow["from"]
		var to_cell: Vector2i = arrow["to"]
		var a: Vector2 = _center(from_cell)
		var b: Vector2 = _center(to_cell)
		var dir: Vector2 = (b - a).normalized()
		var start: Vector2 = a + dir * float(tile) * 0.32
		var end: Vector2 = b - dir * float(tile) * 0.3
		if start.distance_to(end) < 4.0:
			continue
		if DictUtil.to_bool(arrow["dashed"]):
			draw_dashed_line(start, end, COL_ARROW, 2.5, 7.0)
		else:
			draw_line(start, end, COL_ARROW, 2.5)
		var side: Vector2 = dir.orthogonal() * 5.0
		draw_colored_polygon(PackedVector2Array([end + dir * 4.0, end - dir * 7.0 + side, end - dir * 7.0 - side]), COL_ARROW)
