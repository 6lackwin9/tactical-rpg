class_name CombatDockHost
extends Control

# ==============================================================================
# Tactical RPG - Combat Dock Host (production combat UI layout)
# Static dock regions filled from data-driven layout presets. The game UI stays
# fixed; a layout edit mode (⚙ / F9) lets any panel be undocked into a floating
# UiWindow - its region keeps a same-size placeholder so nothing reflows - and
# docked back by closing the window. Also owns the full-screen modals (log,
# timeline, party & stash), keyboard shortcuts and the auto-play timer.
# ==============================================================================

signal layout_changed()

const REGIONS: Array[String] = ["top_left", "top_center", "top_right", "left", "center", "right", "bottom_left", "bottom_center"]
const DEFAULT_LAYOUT: String = "Standard"
const LAYOUTS: Dictionary = {
	"Standard": {
		"top_left": ["party"], "top_center": ["action_log"], "top_right": ["enemies"],
		"left": ["inspector"], "center": ["grid"], "right": ["forecast"],
		"bottom_left": ["playback"], "bottom_center": ["timeline"],
	},
	"Mirrored": {
		"top_left": ["enemies"], "top_center": ["action_log"], "top_right": ["party"],
		"left": ["forecast"], "center": ["grid"], "right": ["inspector"],
		"bottom_left": ["playback"], "bottom_center": ["timeline"],
	},
}
const AUTO_PLAY_INTERVAL: float = 0.55

var session: CombatSession = null
var current_layout: String = DEFAULT_LAYOUT
var log_modal: CombatLogModal
var timeline_modal: TimelineModal
var gear_modal: CharacterEquipmentModal = null

var _panels: Dictionary = {}    # String -> CombatPanel
var _regions: Dictionary = {}   # String -> BoxContainer
var _slots: Dictionary = {}     # String -> MarginContainer (panel + placeholder + edit tag, stacked)
var _windows: Dictionary = {}   # String -> UiWindow (undocked panels)
var _float_layer: Control
var _overlay_layer: UiOverlayLayer
var _edit_bar: PanelContainer
var _layout_option: OptionButton
var _edit_mode: bool = false
var _auto_timer: Timer

func _init() -> void:
	name = "CombatDockHost"
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	theme = UiTheme.tooltip_theme()

	var bg := ColorRect.new()
	bg.color = Color(0.06, 0.08, 0.11, 1.0)
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(bg)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 14)
	margin.add_theme_constant_override("margin_right", 14)
	margin.add_theme_constant_override("margin_top", 4)
	margin.add_theme_constant_override("margin_bottom", 4)
	add_child(margin)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 4)
	margin.add_child(root)

	var top := _row(root, 8)
	top.custom_minimum_size = Vector2(0, 84)
	_add_region(top, "top_left", false)
	_add_region(top, "top_center", true)
	_add_region(top, "top_right", false)

	var center := _row(root, 10)
	center.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_add_region(center, "left", false)
	_add_region(center, "center", true)
	_add_region(center, "right", false)

	var bottom_panel := PanelContainer.new()
	bottom_panel.custom_minimum_size = Vector2(0, 96)
	bottom_panel.add_theme_stylebox_override("panel", UiTheme.box(Color(0.07, 0.09, 0.14, 0.98), Color(0.22, 0.32, 0.46, 0.9), 6, 8, 3))
	root.add_child(bottom_panel)
	var bottom := HBoxContainer.new()
	bottom.add_theme_constant_override("separation", 10)
	bottom_panel.add_child(bottom)
	_add_region(bottom, "bottom_left", false)
	_add_region(bottom, "bottom_center", true)

	_float_layer = Control.new()
	_float_layer.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_float_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_float_layer)

	_overlay_layer = UiOverlayLayer.new()
	add_child(_overlay_layer)

	_build_edit_controls()

	log_modal = CombatLogModal.new()
	add_child(log_modal)
	timeline_modal = TimelineModal.new()
	add_child(timeline_modal)

	_auto_timer = Timer.new()
	_auto_timer.wait_time = AUTO_PLAY_INTERVAL
	_auto_timer.timeout.connect(func() -> void: session.auto_play_tick())
	add_child(_auto_timer)

func _row(parent: Control, separation: int) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", separation)
	parent.add_child(row)
	return row

func _add_region(parent: Control, region: String, expand: bool) -> void:
	var box := VBoxContainer.new()
	box.name = region
	box.add_theme_constant_override("separation", 6)
	if expand:
		box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if region == "center":
		box.alignment = BoxContainer.ALIGNMENT_CENTER
	parent.add_child(box)
	_regions[region] = box

# ------------------------------------------------------------------------------
# Setup
# ------------------------------------------------------------------------------

func setup(p_session: CombatSession) -> void:
	session = p_session
	_overlay_layer.set_session(session)
	var panels: Array[CombatPanel] = [
		CombatRosterPanel.create(true),
		CombatRosterPanel.create(false),
		CombatActionLogPanel.new(),
		CombatInspectorPanel.new(),
		CombatGridPanel.new(),
		CombatForecastPanel.new(),
		CombatPlaybackPanel.new(),
		CombatTimelinePanel.new(),
	]
	for panel: CombatPanel in panels:
		_panels[panel.panel_id] = panel
		_slots[panel.panel_id] = _make_slot(panel)
		panel.modal_requested.connect(open_modal)
		_overlay_layer.host_panel(panel)
		panel.bind(session)

	log_modal.cell_clicked.connect(func(e: ActionLogEntry) -> void: session.select_log_entry(e))
	timeline_modal.action_lookup_fn = session.find_recorded_action
	timeline_modal.token_texture_fn = session.encounter.get_token_texture
	timeline_modal.round_nav_requested.connect(func(delta: int) -> void: session.navigate_round(delta))
	timeline_modal.block_selected.connect(func(b: int) -> void: session.select_block(b))
	timeline_modal.combatant_selected.connect(func(c: Combatant) -> void: session.select_combatant(c))
	session.changed.connect(_on_session_changed)
	apply_layout(DEFAULT_LAYOUT)

func get_panel(panel_id: String) -> CombatPanel:
	var raw: Variant = _panels.get(panel_id, null)
	if raw is CombatPanel:
		var panel: CombatPanel = raw
		return panel
	return null

func panel_ids() -> Array[String]:
	var ids: Array[String] = []
	for id: String in _panels:
		ids.append(id)
	return ids

func _on_session_changed() -> void:
	if session.is_auto_playing and _auto_timer.is_stopped() and is_inside_tree():
		_auto_timer.start()
	elif not session.is_auto_playing and not _auto_timer.is_stopped():
		_auto_timer.stop()
	if log_modal.visible:
		log_modal.set_entries(session.log_entries)
	if timeline_modal.visible:
		_refresh_timeline_modal()

# ------------------------------------------------------------------------------
# Dock slots, layouts and undocking
# ------------------------------------------------------------------------------

# A slot stacks (in the same rect): the panel, its same-size placeholder while
# floating, and the edit-mode undock tag overlaid on the top-right corner, so
# toggling edit mode never moves anything.
func _make_slot(panel: CombatPanel) -> MarginContainer:
	var slot := MarginContainer.new()
	slot.name = "Slot_%s" % panel.panel_id
	slot.add_child(panel)

	var placeholder := PanelContainer.new()
	placeholder.name = "Placeholder"
	placeholder.visible = false
	placeholder.add_theme_stylebox_override("panel", UiTheme.box(Color(0.08, 0.10, 0.14, 0.6), Color(0.3, 0.45, 0.65, 0.6), 6, 8, 6))
	var ph_row := HBoxContainer.new()
	ph_row.alignment = BoxContainer.ALIGNMENT_CENTER
	placeholder.add_child(ph_row)
	ph_row.add_child(UiTheme.label("⇱ %s is floating" % panel.panel_title, UiTheme.FONT_SM, UiTheme.COL_MUTED))
	ph_row.add_child(UiTheme.button("Dock", "Return the panel to this spot", func() -> void: dock(panel.panel_id), 9))
	slot.add_child(placeholder)

	var strip := HBoxContainer.new()
	strip.name = "EditStrip"
	strip.visible = false
	strip.alignment = BoxContainer.ALIGNMENT_END
	strip.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	strip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	slot.add_child(strip)
	var tag := PanelContainer.new()
	tag.add_theme_stylebox_override("panel", UiTheme.box(Color(0.12, 0.10, 0.04, 0.95), UiTheme.COL_WARN, 4, 4, 1, 1))
	strip.add_child(tag)
	var tag_row := HBoxContainer.new()
	tag_row.add_theme_constant_override("separation", 4)
	tag.add_child(tag_row)
	tag_row.add_child(UiTheme.label("%s %s" % [panel.panel_icon, panel.panel_title], 9, UiTheme.COL_WARN))
	tag_row.add_child(UiTheme.button("⇱ Undock", "Float this panel in its own window (close the window to dock it back)", func() -> void: undock(panel.panel_id), 9))
	return slot

func _slot(panel_id: String) -> MarginContainer:
	var raw: Variant = _slots.get(panel_id, null)
	if raw is MarginContainer:
		var slot: MarginContainer = raw
		return slot
	return null

func _region(region: String) -> BoxContainer:
	var raw: Variant = _regions.get(region, null)
	if raw is BoxContainer:
		var box: BoxContainer = raw
		return box
	return null

func apply_layout(layout_name: String) -> void:
	if not LAYOUTS.has(layout_name):
		return
	dock_all()
	current_layout = layout_name
	for id: String in _slots:
		var slot: MarginContainer = _slot(id)
		if slot.get_parent() != null:
			slot.get_parent().remove_child(slot)
	var layout: Dictionary = LAYOUTS[layout_name]
	for region: String in REGIONS:
		var ids_raw: Variant = layout.get(region, [])
		if not (ids_raw is Array):
			continue
		var ids: Array = ids_raw
		for id_raw: Variant in ids:
			var id: String = str(id_raw)
			var slot: MarginContainer = _slot(id)
			if slot == null:
				continue
			_apply_region_flags(slot, region)
			_region(region).add_child(slot)
	if _layout_option != null:
		for i: int in range(_layout_option.item_count):
			if _layout_option.get_item_text(i) == layout_name:
				_layout_option.select(i)
	layout_changed.emit()

func _apply_region_flags(slot: Control, region: String) -> void:
	match region:
		"top_center", "bottom_center":
			slot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		"center":
			slot.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
		_:
			slot.size_flags_horizontal = Control.SIZE_FILL
	var panel_raw: Variant = _panels.get(slot.name.trim_prefix("Slot_"), null)
	if panel_raw is CombatPanel:
		var panel: CombatPanel = panel_raw
		panel.size_flags_horizontal = slot.size_flags_horizontal

func region_of(panel_id: String) -> String:
	var slot: MarginContainer = _slot(panel_id)
	if slot == null or slot.get_parent() == null:
		return ""
	return str(slot.get_parent().name)

func is_undocked(panel_id: String) -> bool:
	return _windows.has(panel_id)

func undock(panel_id: String) -> void:
	var panel: CombatPanel = get_panel(panel_id)
	var slot: MarginContainer = _slot(panel_id)
	if panel == null or slot == null or is_undocked(panel_id):
		return
	var keep_size: Vector2 = panel.size if panel.size != Vector2.ZERO else panel.get_combined_minimum_size()
	slot.remove_child(panel)
	var placeholder: Control = slot.get_node("Placeholder") as Control
	placeholder.custom_minimum_size = keep_size
	placeholder.visible = true

	var win := UiWindow.new()
	win.setup(panel)
	win.close_requested.connect(func(_w: UiWindow) -> void: dock(panel_id))
	_float_layer.add_child(win)
	win.size = (keep_size + Vector2(4, UiWindow.TITLE_HEIGHT + 4)).max(win.custom_minimum_size)
	var origin: Vector2 = slot.get_global_rect().position - get_global_rect().position + Vector2(24, 24)
	win.position = origin.clamp(Vector2.ZERO, (size - win.size).max(Vector2.ZERO))
	_windows[panel_id] = win
	layout_changed.emit()

func dock(panel_id: String) -> void:
	var win_raw: Variant = _windows.get(panel_id, null)
	var panel: CombatPanel = get_panel(panel_id)
	var slot: MarginContainer = _slot(panel_id)
	if not (win_raw is UiWindow) or panel == null or slot == null:
		return
	var win: UiWindow = win_raw
	_windows.erase(panel_id)
	if panel.get_parent() != null:
		panel.get_parent().remove_child(panel)
	panel.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	slot.add_child(panel)
	slot.move_child(panel, 0)
	var placeholder: Control = slot.get_node("Placeholder") as Control
	placeholder.visible = false
	if win.get_parent() != null:
		win.get_parent().remove_child(win)
	win.queue_free()
	panel.refresh()
	layout_changed.emit()

func dock_all() -> void:
	for id: String in _windows.keys():
		dock(id)

# ------------------------------------------------------------------------------
# Layout edit mode (debug / customization; the game UI itself stays static)
# ------------------------------------------------------------------------------

func _build_edit_controls() -> void:
	var corner := Button.new()
	corner.text = "⚙"
	corner.tooltip_text = "Layout edit mode [F9]: undock panels, switch layout presets"
	corner.focus_mode = Control.FOCUS_NONE
	corner.flat = true
	corner.add_theme_font_size_override("font_size", 11)
	corner.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT)
	corner.offset_left = -26
	corner.offset_right = -2
	corner.offset_top = 0
	corner.offset_bottom = 20
	corner.pressed.connect(func() -> void: set_edit_mode(not _edit_mode))
	add_child(corner)

	_edit_bar = PanelContainer.new()
	_edit_bar.visible = false
	_edit_bar.set_anchors_and_offsets_preset(Control.PRESET_CENTER_TOP)
	_edit_bar.grow_horizontal = Control.GROW_DIRECTION_BOTH
	_edit_bar.offset_top = 2
	_edit_bar.add_theme_stylebox_override("panel", UiTheme.box(Color(0.12, 0.10, 0.04, 0.97), UiTheme.COL_WARN, 6, 10, 4, 1))
	add_child(_edit_bar)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	_edit_bar.add_child(row)
	row.add_child(UiTheme.label("LAYOUT EDIT", UiTheme.FONT_SM, UiTheme.COL_WARN))
	_layout_option = OptionButton.new()
	_layout_option.focus_mode = Control.FOCUS_NONE
	_layout_option.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	for layout_name: String in LAYOUTS:
		_layout_option.add_item(layout_name)
	_layout_option.item_selected.connect(func(index: int) -> void: apply_layout(_layout_option.get_item_text(index)))
	row.add_child(_layout_option)
	row.add_child(UiTheme.button("Dock All", "Return every floating panel to its dock", dock_all, UiTheme.FONT_SM))
	row.add_child(UiTheme.button("✔ Done", "Leave layout edit mode", func() -> void: set_edit_mode(false), UiTheme.FONT_SM))

func set_edit_mode(enabled: bool) -> void:
	_edit_mode = enabled
	_edit_bar.visible = enabled
	for id: String in _slots:
		var slot: MarginContainer = _slot(id)
		var strip: Control = slot.get_node("EditStrip") as Control
		strip.visible = enabled

func is_edit_mode() -> bool:
	return _edit_mode

# ------------------------------------------------------------------------------
# Modals
# ------------------------------------------------------------------------------

func open_modal(kind: String) -> void:
	match kind:
		"log":
			log_modal.set_entries(session.log_entries)
			log_modal.open_modal()
		"timeline":
			timeline_modal.open_modal()
			_refresh_timeline_modal()
		"gear":
			_open_gear_modal()

func close_modals() -> bool:
	if log_modal.visible:
		log_modal.close_modal()
		return true
	if timeline_modal.visible:
		timeline_modal.close_modal()
		return true
	return false

func _refresh_timeline_modal() -> void:
	timeline_modal.update_view(
		session.viewed_round,
		session.playhead(),
		not session.is_live_view(),
		session.allies,
		session.enemies_in_view(),
		session.display_snapshot(),
		session.can_view_prev_round(),
		session.can_view_next_round()
	)

# The Party & Stash screen is built on first use (it syncs gear into unit stats).
func _open_gear_modal() -> void:
	if gear_modal == null:
		gear_modal = CharacterEquipmentModal.new()
		gear_modal.equipment_updated.connect(func(_c: Combatant) -> void: session.notify_external_change())
		add_child(gear_modal)
		gear_modal.setup_heroes(session.allies, session.encounter.party_stash)
	var hero: Combatant = session.selected_combatant if session.is_ally(session.selected_combatant) else session.focused_hero
	gear_modal.open_modal(hero)

# ------------------------------------------------------------------------------
# Keyboard shortcuts
# ------------------------------------------------------------------------------

func _unhandled_input(event: InputEvent) -> void:
	var key := event as InputEventKey
	if key == null or not key.pressed or key.echo or session == null:
		return
	var handled: bool = true
	match key.keycode:
		KEY_ESCAPE:
			handled = close_modals()
		KEY_L:
			if log_modal.visible:
				log_modal.close_modal()
			else:
				open_modal("log")
		KEY_F, KEY_TAB:
			if timeline_modal.visible:
				timeline_modal.close_modal()
			else:
				open_modal("timeline")
		KEY_G:
			open_modal("gear")
		KEY_F9:
			set_edit_mode(not _edit_mode)
		KEY_BRACKETLEFT:
			session.log_prev()
		KEY_BRACKETRIGHT:
			session.log_next()
		KEY_Q, KEY_LEFT:
			session.select_block(session.playhead() - 1)
		KEY_E, KEY_RIGHT:
			session.select_block(session.playhead() + 1)
		KEY_SPACE:
			session.toggle_auto_play()
		_:
			handled = false
	if handled:
		get_viewport().set_input_as_handled()
