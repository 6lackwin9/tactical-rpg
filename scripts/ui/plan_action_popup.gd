class_name PlanActionPopup
extends PanelContainer

# ==============================================================================
# Tactical RPG - Plan Action Popup (the one small input for planning)
# Opens next to the clicked target (an enemy, yourself, or a tile) and lists only
# the actions that make sense for it, each with its AP and block cost. Hovering
# a row previews it on the grid; clicking (or pressing its number) appends it to
# the hero's sequence. Enter / Space / clicking the same target again takes the
# first (default) action. Esc or clicking elsewhere closes it.
# ==============================================================================

signal option_chosen(step: PlanStep)
signal option_hovered(step: PlanStep)   # null when the pointer leaves a row
signal closed()

var _options: Array[PlanOption] = []   # the valid ones, in row order
var _target_rect: Rect2 = Rect2()       # the clicked target: pressing it again is handled by the opener
var _title_lbl: Label
var _list: VBoxContainer

func _init() -> void:
	name = "PlanActionPopup"
	top_level = true
	visible = false
	z_index = 100
	mouse_filter = Control.MOUSE_FILTER_STOP
	add_theme_stylebox_override("panel", UiTheme.box(Color(0.05, 0.08, 0.13, 0.98), UiTheme.COL_INFO, 5, 6, 5, 2))
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 3)
	add_child(root)
	_title_lbl = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_HEADER)
	root.add_child(_title_lbl)
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 2)
	root.add_child(_list)

# `options` come from CombatPlan (valid or not); only the valid ones become rows.
func open_for(title: String, options: Array[PlanOption], anchor: Vector2, target_rect: Rect2 = Rect2()) -> void:
	_target_rect = target_rect
	_options = []
	UiTheme.clear_children(_list)
	_title_lbl.text = title
	var first_reason: String = ""
	for o: PlanOption in options:
		if o.valid:
			_options.append(o)
		elif first_reason.is_empty():
			first_reason = o.reason
	for i: int in range(_options.size()):
		_list.add_child(_make_row(i, _options[i]))
	if _options.is_empty():
		var why: String = first_reason if not first_reason.is_empty() else "Nothing to do here."
		var lbl: Label = UiTheme.label(why, 9, UiTheme.COL_MUTED)
		lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		lbl.custom_minimum_size = Vector2(170, 0)
		_list.add_child(lbl)
	visible = true
	reset_size()
	_place.call_deferred(anchor)

func default_step() -> PlanStep:
	return _options[0].step if not _options.is_empty() else null

func close() -> void:
	if visible:
		visible = false
		option_hovered.emit(null)
		closed.emit()

func _place(anchor: Vector2) -> void:
	var view: Vector2 = get_viewport_rect().size
	var pos: Vector2 = anchor
	if pos.x + size.x > view.x - 6.0:
		pos.x = maxf(6.0, anchor.x - size.x - 70.0)
	if pos.y + size.y > view.y - 6.0:
		pos.y = maxf(6.0, view.y - size.y - 6.0)
	global_position = pos

func _make_row(index: int, option: PlanOption) -> Button:
	var step: PlanStep = option.step
	var text: String = "%d  %s" % [index + 1, _row_text(option)]
	var btn: Button = UiTheme.button(text, option.cost_text(), func() -> void: _choose(step), 10)
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	btn.custom_minimum_size = Vector2(190, 0)
	if not option.warning.is_empty():
		btn.modulate = Color(1.0, 0.8, 0.4)
	btn.mouse_entered.connect(func() -> void: option_hovered.emit(step))
	btn.mouse_exited.connect(func() -> void: option_hovered.emit(null))
	return btn

static func _row_text(option: PlanOption) -> String:
	var step: PlanStep = option.step
	var name_text: String = step.skill.label() if step.skill != null else "🏃 Withdraw"
	if step.mode == PlanStep.Mode.GOTO:
		name_text = "➜ Move here"
	var cost: String = ("+%d AP" % -option.ap) if option.ap < 0 else ("%d AP" % option.ap)
	return "%s   %s · %db" % [name_text, cost, option.blocks]

func _choose(step: PlanStep) -> void:
	visible = false
	option_hovered.emit(null)
	option_chosen.emit(step)
	closed.emit()

func _input(event: InputEvent) -> void:
	if not visible:
		return
	var key := event as InputEventKey
	if key != null and key.pressed and not key.echo:
		if key.keycode == KEY_ESCAPE:
			close()
			get_viewport().set_input_as_handled()
		elif key.keycode == KEY_ENTER or key.keycode == KEY_KP_ENTER or key.keycode == KEY_SPACE:
			var step: PlanStep = default_step()
			if step != null:
				_choose(step)
			else:
				close()
			get_viewport().set_input_as_handled()
		elif key.keycode >= KEY_1 and key.keycode <= KEY_9:
			var idx: int = key.keycode - KEY_1
			if idx < _options.size():
				_choose(_options[idx].step)
				get_viewport().set_input_as_handled()
		return
	var mouse := event as InputEventMouseButton
	if mouse != null and mouse.pressed and not get_global_rect().has_point(mouse.global_position) and not _target_rect.has_point(mouse.global_position):
		close()   # the click still reaches whatever is underneath
