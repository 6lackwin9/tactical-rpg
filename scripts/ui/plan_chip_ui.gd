class_name PlanChipUI
extends PanelContainer

# ==============================================================================
# Tactical RPG - Plan Chip (one step of a unit's sequence in the Plan Round screen)
# Shows the step, its cost and any error, can be removed with ✕, and can be
# dragged onto another chip of the same unit to reorder the sequence.
# ==============================================================================

signal remove_requested(actor: Combatant, index: int)
signal reorder_requested(actor: Combatant, from_index: int, to_index: int)

var actor: Combatant = null
var step_index: int = 0

static func create(p_actor: Combatant, p_index: int, step: PlanStep, blocks: int, ap: int, error_text: String, block_w: int) -> PlanChipUI:
	var chip := PlanChipUI.new()
	chip.actor = p_actor
	chip.step_index = p_index
	var invalid: bool = not error_text.is_empty()
	var col: Color = _step_color(step)
	chip.custom_minimum_size = Vector2(maxi(1, blocks) * block_w + (blocks - 1) * 4 if not invalid else 120, 40)
	chip.add_theme_stylebox_override("panel", UiTheme.box(Color(0.35, 0.1, 0.1, 0.95) if invalid else col.darkened(0.55), UiTheme.COL_BAD if invalid else col, 4, 5, 2))
	chip.mouse_default_cursor_shape = Control.CURSOR_DRAG
	chip.tooltip_text = ("⚠ %s" % error_text) if invalid else "%s\n%d block(s) • %d AP\nDrag to reorder • ✕ to remove" % [step.title(), blocks, ap]
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 2)
	chip.add_child(row)
	var col_box := VBoxContainer.new()
	col_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col_box.add_theme_constant_override("separation", 0)
	col_box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(col_box)
	var title: Label = UiTheme.label(step.title(), 9, Color.WHITE)
	title.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	title.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col_box.add_child(title)
	var sub: Label = UiTheme.label(("⚠ " + error_text) if invalid else "%db • %d AP" % [blocks, ap], 8, UiTheme.COL_BAD if invalid else UiTheme.COL_MUTED)
	sub.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	sub.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col_box.add_child(sub)
	var close: Button = UiTheme.button("✕", "Remove this step", func() -> void: chip.remove_requested.emit(p_actor, p_index), 8)
	close.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	row.add_child(close)
	return chip

static func _step_color(step: PlanStep) -> Color:
	if step.skill == null:
		return Color(0.3, 0.85, 0.5)  # withdraw
	match step.skill.kind:
		SkillDef.Kind.ATTACK:
			return Color(0.9, 0.3, 0.3)
		SkillDef.Kind.DEFEND:
			return Color(0.3, 0.6, 0.95)
		SkillDef.Kind.FOCUS:
			return Color(0.7, 0.4, 0.95)
		SkillDef.Kind.MOVE:
			return Color(0.2, 0.8, 0.9)
	return Color(0.7, 0.7, 0.8)

func _get_drag_data(_at_position: Vector2) -> Variant:
	var preview := Label.new()
	preview.text = "  ↕ step %d" % (step_index + 1)
	set_drag_preview(preview)
	return {"actor": actor, "index": step_index}

func _can_drop_data(_at_position: Vector2, data: Variant) -> bool:
	if not (data is Dictionary):
		return false
	var d: Dictionary = data
	return d.get("actor", null) == actor and d.get("index", -1) != step_index

func _drop_data(_at_position: Vector2, data: Variant) -> void:
	if data is Dictionary:
		var d: Dictionary = data
		reorder_requested.emit(actor, DictUtil.int_of(d, "index"), step_index)
