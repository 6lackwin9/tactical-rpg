class_name DojoWindow
extends Control

# ==============================================================================
# Tactical RPG - Dojo Floating Window
# Draggable, resizable, collapsible window that hosts one DojoPanel. Position
# and size are tracked as fractions of the desk (frac_rect) by the workspace so
# layouts survive window resizes.
# ==============================================================================

signal close_requested(window: DojoWindow)
signal raise_requested(window: DojoWindow)
signal geometry_committed(window: DojoWindow)

const TITLE_HEIGHT: float = 26.0
const FRAME_BORDER: float = 1.0
const GRIP_SIZE: float = 16.0

var panel: DojoPanel = null
var window_id: String = ""
var frac_rect: Rect2 = Rect2(0, 0, 0.3, 0.5)
var is_collapsed: bool = false

var _frame: PanelContainer
var _title_bar: PanelContainer
var _title_lbl: Label
var _collapse_btn: Button
var _body: Control
var _grip: Control
var _dragging: bool = false
var _resizing: bool = false
var _min_expanded: Vector2 = Vector2(220, 120)

func _init() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP

func setup(p_panel: DojoPanel) -> void:
	panel = p_panel
	window_id = p_panel.panel_id
	_min_expanded = p_panel.default_min_size.max(Vector2(200, 100))
	custom_minimum_size = _min_expanded
	_build()

func _build() -> void:
	_frame = PanelContainer.new()
	_frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_frame.add_theme_stylebox_override("panel", DojoTheme.box(DojoTheme.COL_PANEL_BG, Color(0.28, 0.42, 0.62, 0.95), 6, int(FRAME_BORDER), int(FRAME_BORDER), 1))
	_frame.mouse_filter = Control.MOUSE_FILTER_PASS
	add_child(_frame)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 0)
	_frame.add_child(vbox)

	_title_bar = PanelContainer.new()
	_title_bar.custom_minimum_size = Vector2(0, TITLE_HEIGHT)
	_title_bar.mouse_default_cursor_shape = Control.CURSOR_MOVE
	var title_style := DojoTheme.box(Color(0.13, 0.19, 0.28, 1.0), Color(0, 0, 0, 0), 5, 8, 2, 0)
	title_style.corner_radius_bottom_left = 0
	title_style.corner_radius_bottom_right = 0
	_title_bar.add_theme_stylebox_override("panel", title_style)
	_title_bar.gui_input.connect(_on_title_input)
	vbox.add_child(_title_bar)

	var title_row := HBoxContainer.new()
	title_row.add_theme_constant_override("separation", 4)
	title_row.mouse_filter = Control.MOUSE_FILTER_PASS
	_title_bar.add_child(title_row)

	_title_lbl = DojoTheme.label("%s  %s" % [panel.panel_icon, panel.panel_title], DojoTheme.FONT_MD, DojoTheme.COL_TITLE)
	_title_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_title_lbl.mouse_filter = Control.MOUSE_FILTER_PASS
	title_row.add_child(_title_lbl)

	_collapse_btn = _make_title_button("–", "Collapse / expand (or double-click the title)", _toggle_collapse)
	title_row.add_child(_collapse_btn)
	title_row.add_child(_make_title_button("✕", "Close window (reopen from the toolbar)", func() -> void: close_requested.emit(self)))

	_body = Control.new()
	_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_body.clip_contents = true
	vbox.add_child(_body)

	panel.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_body.add_child(panel)

	_grip = Control.new()
	_grip.custom_minimum_size = Vector2(GRIP_SIZE, GRIP_SIZE)
	_grip.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_RIGHT)
	_grip.offset_left = -GRIP_SIZE - 1.0
	_grip.offset_top = -GRIP_SIZE - 1.0
	_grip.offset_right = -1.0
	_grip.offset_bottom = -1.0
	_grip.mouse_default_cursor_shape = Control.CURSOR_FDIAGSIZE
	_grip.mouse_filter = Control.MOUSE_FILTER_STOP
	_grip.draw.connect(_draw_grip)
	_grip.gui_input.connect(_on_grip_input)
	add_child(_grip)

func _make_title_button(text: String, tip: String, callback: Callable) -> Button:
	var btn: Button = DojoTheme.button(text, tip, callback, DojoTheme.FONT_SM)
	btn.custom_minimum_size = Vector2(20, 18)
	btn.flat = true
	return btn

func _draw_grip() -> void:
	var col := Color(0.55, 0.7, 0.9, 0.8)
	_grip.draw_line(Vector2(GRIP_SIZE - 3, 4), Vector2(4, GRIP_SIZE - 3), col, 1.0)
	_grip.draw_line(Vector2(GRIP_SIZE - 3, 8), Vector2(8, GRIP_SIZE - 3), col, 1.0)
	_grip.draw_line(Vector2(GRIP_SIZE - 3, 12), Vector2(12, GRIP_SIZE - 3), col, 1.0)

# ------------------------------------------------------------------------------
# Geometry
# ------------------------------------------------------------------------------

func collapsed_height() -> float:
	return TITLE_HEIGHT + FRAME_BORDER * 2.0

func set_collapsed(collapsed: bool) -> void:
	is_collapsed = collapsed
	_body.visible = not collapsed
	_grip.visible = not collapsed
	_collapse_btn.text = "▢" if collapsed else "–"
	custom_minimum_size = Vector2(_min_expanded.x, collapsed_height()) if collapsed else _min_expanded

func apply_fraction(desk_size: Vector2) -> void:
	var target_pos: Vector2 = frac_rect.position * desk_size
	var target_size: Vector2 = frac_rect.size * desk_size
	position = target_pos
	size = Vector2(target_size.x, collapsed_height() if is_collapsed else target_size.y).max(custom_minimum_size)
	_clamp_to_parent()

func commit_fraction() -> void:
	var desk: Vector2 = get_parent_area_size()
	if desk.x <= 0.0 or desk.y <= 0.0:
		return
	var height: float = size.y
	if is_collapsed:
		height = maxf(frac_rect.size.y * desk.y, custom_minimum_size.y)
	frac_rect = Rect2(position / desk, Vector2(size.x, height) / desk)
	geometry_committed.emit(self)

func _clamp_to_parent() -> void:
	var desk: Vector2 = get_parent_area_size()
	if desk.x <= 0.0 or desk.y <= 0.0:
		return
	size = size.min(desk).max(custom_minimum_size)
	position = Vector2(
		clampf(position.x, 0.0, maxf(0.0, desk.x - size.x)),
		clampf(position.y, 0.0, maxf(0.0, desk.y - size.y))
	)

func _toggle_collapse() -> void:
	if not is_collapsed:
		frac_rect.size.y = size.y / maxf(1.0, get_parent_area_size().y)
	set_collapsed(not is_collapsed)
	apply_fraction(get_parent_area_size())
	commit_fraction()

# ------------------------------------------------------------------------------
# Input: drag by title bar, resize by grip
# ------------------------------------------------------------------------------

func _on_title_input(event: InputEvent) -> void:
	var button := event as InputEventMouseButton
	if button != null and button.button_index == MOUSE_BUTTON_LEFT:
		_dragging = button.pressed
		if button.pressed:
			raise_requested.emit(self)
			if button.double_click:
				_dragging = false
				_toggle_collapse()
		elif not button.pressed:
			commit_fraction()
		return
	var motion := event as InputEventMouseMotion
	if motion != null and _dragging:
		position += motion.relative
		_clamp_to_parent()

func _on_grip_input(event: InputEvent) -> void:
	var button := event as InputEventMouseButton
	if button != null and button.button_index == MOUSE_BUTTON_LEFT:
		_resizing = button.pressed
		if button.pressed:
			raise_requested.emit(self)
		else:
			commit_fraction()
		return
	var motion := event as InputEventMouseMotion
	if motion != null and _resizing:
		size = (size + motion.relative).max(custom_minimum_size)
		_clamp_to_parent()
