class_name DojoWorkspace
extends Control

# ==============================================================================
# Tactical RPG - Dojo Workspace
# Toolbar + free-form desk hosting one DojoWindow per DojoPanel. Handles active
# unit switching, window toggles, layout presets and layout persistence.
# ==============================================================================

signal stash_requested()
signal reset_requested()

const LAYOUT_PATH: String = "user://gear_dojo_layout.cfg"
const GAP: float = 3.0
const DEFAULT_LAYOUT: String = "Workbench"

# Layout presets: panel_id -> Rect2 (fractions of the desk). Missing id = hidden.
const LAYOUTS: Dictionary = {
	"Workbench": {
		"inspector": Rect2(0.00, 0.00, 0.21, 0.55),
		"derived": Rect2(0.00, 0.55, 0.21, 0.45),
		"gear": Rect2(0.21, 0.00, 0.25, 1.00),
		"stage": Rect2(0.46, 0.00, 0.29, 0.50),
		"console": Rect2(0.46, 0.50, 0.29, 0.50),
		"log": Rect2(0.75, 0.00, 0.25, 1.00),
	},
	"Combat Focus": {
		"stage": Rect2(0.00, 0.00, 0.50, 0.62),
		"console": Rect2(0.00, 0.62, 0.50, 0.38),
		"log": Rect2(0.50, 0.00, 0.50, 0.62),
		"derived": Rect2(0.50, 0.62, 0.25, 0.38),
		"inspector": Rect2(0.75, 0.62, 0.25, 0.38),
	},
	"Gear Focus": {
		"gear": Rect2(0.00, 0.00, 0.34, 1.00),
		"inspector": Rect2(0.34, 0.00, 0.26, 0.55),
		"derived": Rect2(0.34, 0.55, 0.26, 0.45),
		"stage": Rect2(0.60, 0.00, 0.40, 0.42),
		"console": Rect2(0.60, 0.42, 0.40, 0.30),
		"log": Rect2(0.60, 0.72, 0.40, 0.28),
	},
}

var session: DojoSession = null

var _hero_row: HBoxContainer
var _toggle_row: HBoxContainer
var _layout_option: OptionButton
var _desk: Control
var _windows: Dictionary = {}      # String -> DojoWindow
var _toggle_btns: Dictionary = {}  # String -> Button
var _layout_ready: bool = false
var _current_layout: String = DEFAULT_LAYOUT

func _init() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	theme = DojoTheme.tooltip_theme()

	var bg := ColorRect.new()
	bg.color = DojoTheme.COL_APP_BG
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(bg)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	for side: String in ["left", "right", "top", "bottom"]:
		margin.add_theme_constant_override("margin_%s" % side, 6)
	add_child(margin)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 6)
	margin.add_child(root)

	root.add_child(_build_toolbar())

	_desk = Control.new()
	_desk.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_desk.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_desk.clip_contents = true
	_desk.resized.connect(_on_desk_resized)
	root.add_child(_desk)

func _build_toolbar() -> Control:
	var bar := PanelContainer.new()
	bar.add_theme_stylebox_override("panel", DojoTheme.box(Color(0.08, 0.11, 0.16, 0.98), DojoTheme.COL_BORDER, 6, 10, 6))

	# Groups are plain HBoxes inside one wrapping flow, so the bar stays a single
	# row when wide and wraps whole groups (never individual buttons) when narrow.
	var flow := HFlowContainer.new()
	flow.add_theme_constant_override("h_separation", 14)
	flow.add_theme_constant_override("v_separation", 4)
	bar.add_child(flow)

	var title := DojoTheme.label("🥋 GEAR & SKILL DOJO", DojoTheme.FONT_LG, DojoTheme.COL_TITLE)
	title.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	flow.add_child(title)

	var hero_group := _toolbar_group(flow, "Unit:")
	_hero_row = HBoxContainer.new()
	_hero_row.add_theme_constant_override("separation", 3)
	hero_group.add_child(_hero_row)

	var win_group := _toolbar_group(flow, "Windows:")
	_toggle_row = HBoxContainer.new()
	_toggle_row.add_theme_constant_override("separation", 3)
	win_group.add_child(_toggle_row)

	var layout_group := _toolbar_group(flow, "Layout:")
	_layout_option = OptionButton.new()
	_layout_option.focus_mode = Control.FOCUS_NONE
	_layout_option.fit_to_longest_item = false
	_layout_option.custom_minimum_size = Vector2(120, 0)
	_layout_option.add_theme_font_size_override("font_size", DojoTheme.FONT_SM)
	for layout_name: String in LAYOUTS:
		_layout_option.add_item(layout_name)
	_layout_option.item_selected.connect(_on_layout_selected)
	layout_group.add_child(_layout_option)
	layout_group.add_child(DojoTheme.button("⟲ Re-tile", "Re-apply the selected layout and reopen all of its windows", func() -> void: apply_layout(_current_layout)))

	var action_group := _toolbar_group(flow, "")
	action_group.add_child(DojoTheme.button("📦 Stash", "Open the full stash & character equipment screen", func() -> void: stash_requested.emit()))
	action_group.add_child(DojoTheme.button("↺ Reset Dojo", "Restore heroes, loadouts, stash, dummy and log to baseline", func() -> void: reset_requested.emit()))
	return bar

func _toolbar_group(parent: Control, caption: String) -> HBoxContainer:
	var group := HBoxContainer.new()
	group.add_theme_constant_override("separation", 4)
	group.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	parent.add_child(group)
	if not caption.is_empty():
		group.add_child(DojoTheme.label(caption, DojoTheme.FONT_SM, DojoTheme.COL_MUTED))
	return group

# ------------------------------------------------------------------------------
# Setup
# ------------------------------------------------------------------------------

func setup(p_session: DojoSession, panels: Array[DojoPanel]) -> void:
	session = p_session
	session.changed.connect(_refresh_hero_buttons)
	for panel: DojoPanel in panels:
		_add_panel(panel)
	_refresh_hero_buttons()

func get_panel(panel_id: String) -> DojoPanel:
	var win_raw: Variant = _windows.get(panel_id, null)
	if win_raw is DojoWindow:
		var win: DojoWindow = win_raw
		return win.panel
	return null

func _add_panel(panel: DojoPanel) -> void:
	panel.bind(session)

	var win := DojoWindow.new()
	win.setup(panel)
	win.visible = false
	win.close_requested.connect(_on_window_close)
	win.raise_requested.connect(_on_window_raise)
	win.geometry_committed.connect(_on_geometry_committed)
	_desk.add_child(win)
	_windows[panel.panel_id] = win

	var short_title: String = panel.panel_short_title if not panel.panel_short_title.is_empty() else panel.panel_title
	var toggle: Button = DojoTheme.button("%s %s" % [panel.panel_icon, short_title], "Show / hide the %s window" % panel.panel_title, func() -> void: _on_toggle_pressed(panel.panel_id))
	toggle.toggle_mode = true
	_toggle_row.add_child(toggle)
	_toggle_btns[panel.panel_id] = toggle

func _refresh_hero_buttons() -> void:
	if session == null:
		return
	DojoTheme.clear_children(_hero_row)
	for hero: Combatant in session.party_allies:
		var btn: Button = DojoTheme.button("%s (Lv %d)" % [hero.name, hero.stats.level if hero.stats != null else 1], "Inspect and test with %s" % hero.name, func() -> void: session.set_active_hero(hero))
		btn.toggle_mode = true
		btn.button_pressed = hero == session.active_hero
		if hero == session.active_hero:
			btn.add_theme_color_override("font_color", DojoTheme.COL_GOOD)
		_hero_row.add_child(btn)

# ------------------------------------------------------------------------------
# Window management
# ------------------------------------------------------------------------------

func _on_toggle_pressed(panel_id: String) -> void:
	var win_raw: Variant = _windows.get(panel_id, null)
	if not (win_raw is DojoWindow):
		return
	var win: DojoWindow = win_raw
	win.visible = not win.visible
	if win.visible:
		win.move_to_front()
	_sync_toggle(panel_id)
	save_layout()

func _on_window_close(win: DojoWindow) -> void:
	win.visible = false
	_sync_toggle(win.window_id)
	save_layout()

func _on_window_raise(win: DojoWindow) -> void:
	win.move_to_front()

func _on_geometry_committed(_win: DojoWindow) -> void:
	save_layout()

func _sync_toggle(panel_id: String) -> void:
	var btn_raw: Variant = _toggle_btns.get(panel_id, null)
	var win_raw: Variant = _windows.get(panel_id, null)
	if btn_raw is Button and win_raw is DojoWindow:
		var btn: Button = btn_raw
		var win: DojoWindow = win_raw
		btn.set_pressed_no_signal(win.visible)

func _input(event: InputEvent) -> void:
	var button := event as InputEventMouseButton
	if button == null or not button.pressed or _desk == null:
		return
	var local: Vector2 = _desk.get_local_mouse_position()
	var children: Array[Node] = _desk.get_children()
	for i: int in range(children.size() - 1, -1, -1):
		var node: Node = children[i]
		if node is DojoWindow:
			var win: DojoWindow = node
			if win.visible and Rect2(win.position, win.size).has_point(local):
				if i != children.size() - 1:
					win.move_to_front()
				return

# ------------------------------------------------------------------------------
# Layouts & persistence
# ------------------------------------------------------------------------------

func _on_desk_resized() -> void:
	if _desk.size.x < 50.0 or _desk.size.y < 50.0:
		return
	if not _layout_ready:
		_layout_ready = true
		if not _load_layout():
			apply_layout(DEFAULT_LAYOUT)
		return
	for win_raw: Variant in _windows.values():
		if win_raw is DojoWindow:
			var win: DojoWindow = win_raw
			win.apply_fraction(_desk.size)

func _on_layout_selected(index: int) -> void:
	apply_layout(_layout_option.get_item_text(index))

func apply_layout(layout_name: String) -> void:
	if not LAYOUTS.has(layout_name):
		return
	_current_layout = layout_name
	for i: int in range(_layout_option.item_count):
		if _layout_option.get_item_text(i) == layout_name:
			_layout_option.select(i)

	var layout: Dictionary = LAYOUTS[layout_name]
	for panel_id: String in _windows:
		var win_raw: Variant = _windows[panel_id]
		if not (win_raw is DojoWindow):
			continue
		var win: DojoWindow = win_raw
		win.set_collapsed(false)
		var rect_raw: Variant = layout.get(panel_id, null)
		if rect_raw is Rect2:
			var rect: Rect2 = rect_raw
			win.frac_rect = _inset(rect)
			win.visible = true
			win.apply_fraction(_desk.size)
		else:
			win.visible = false
		_sync_toggle(panel_id)
	save_layout()

# Insets a tile by half a gap on each side so adjacent windows don't touch.
func _inset(rect: Rect2) -> Rect2:
	var desk: Vector2 = _desk.size
	if desk.x <= 0.0 or desk.y <= 0.0:
		return rect
	var pad: Vector2 = Vector2(GAP, GAP) / desk
	return Rect2(rect.position + pad * 0.5, rect.size - pad)

func save_layout() -> void:
	if not _layout_ready:
		return
	var cfg := ConfigFile.new()
	cfg.set_value("layout", "name", _current_layout)
	for panel_id: String in _windows:
		var win_raw: Variant = _windows[panel_id]
		if win_raw is DojoWindow:
			var win: DojoWindow = win_raw
			cfg.set_value(panel_id, "rect", win.frac_rect)
			cfg.set_value(panel_id, "visible", win.visible)
			cfg.set_value(panel_id, "collapsed", win.is_collapsed)
	cfg.save(LAYOUT_PATH)

func _load_layout() -> bool:
	var cfg := ConfigFile.new()
	if cfg.load(LAYOUT_PATH) != OK:
		return false
	for panel_id: String in _windows:
		if not cfg.has_section(panel_id):
			return false
	var name_raw: Variant = cfg.get_value("layout", "name", DEFAULT_LAYOUT)
	_current_layout = name_raw if name_raw is String and LAYOUTS.has(name_raw) else DEFAULT_LAYOUT
	for i: int in range(_layout_option.item_count):
		if _layout_option.get_item_text(i) == _current_layout:
			_layout_option.select(i)

	for panel_id: String in _windows:
		var win_raw: Variant = _windows[panel_id]
		if not (win_raw is DojoWindow):
			continue
		var win: DojoWindow = win_raw
		var rect_raw: Variant = cfg.get_value(panel_id, "rect", Rect2(0, 0, 0.3, 0.5))
		var vis_raw: Variant = cfg.get_value(panel_id, "visible", true)
		var col_raw: Variant = cfg.get_value(panel_id, "collapsed", false)
		var collapsed: bool = col_raw if col_raw is bool else false
		win.frac_rect = rect_raw if rect_raw is Rect2 else Rect2(0, 0, 0.3, 0.5)
		win.set_collapsed(collapsed)
		win.visible = vis_raw if vis_raw is bool else true
		win.apply_fraction(_desk.size)
		_sync_toggle(panel_id)
	return true
