class_name CombatDockHost
extends Control

# ==============================================================================
# Tactical RPG - Combat Dock Host (production combat UI layout)
# COMBAT-UI2-L1 layout C: timeline row on top, a slim party column (the whole height) left, the biggest grid in the
# middle, a slim hostile column (+ forecast) right, playback buttons and the one-line log ticker at the bottom;
# every size comes from CombatHudMetrics (_fit). No gear button or equipment modal in battle. The unit hover card
# (COMBAT-UI2-L2) floats inside the grid panel; Esc closes a pinned card before anything else.
# Static dock regions filled from data-driven layout presets. The game UI stays
# fixed; a layout edit mode (⚙ / F9) lets any panel be undocked into a floating
# UiWindow - its region keeps a same-size placeholder so nothing reflows - and
# docked back by closing the window. Also owns the full-screen modals (log,
# timeline, party & stash), keyboard shortcuts and the auto-play timer.
# ==============================================================================

signal layout_changed()

const REGIONS: Array[String] = ["top", "left_top", "left", "center", "right_top", "right", "bottom_left", "bottom_center"]
const DEFAULT_LAYOUT: String = "Standard"
const LAYOUTS: Dictionary = {
	"Standard": {
		"top": ["timeline"],
		"left_top": ["party"], "left": [], "center": ["grid"], "right_top": ["enemies"], "right": ["forecast"],
		"bottom_left": ["playback"], "bottom_center": ["action_log"],
	},
	"Mirrored": {
		"top": ["timeline"],
		"left_top": ["enemies"], "left": ["forecast"], "center": ["grid"], "right_top": ["party"], "right": [],
		"bottom_left": ["playback"], "bottom_center": ["action_log"],
	},
}
const CORE_COG_PX: int = 32   # the settings cog (UIR-5: 32 px bordered button)
const AUTO_PLAY_INTERVAL: float = 0.55
const AUTOPLAY_TICK: float = 0.3

var session: CombatSession = null
var keys_blocked: bool = false   # BATTLE-FLOW: set while the results popup covers the arena
var current_layout: String = DEFAULT_LAYOUT
var log_modal: CombatLogModal
var timeline_modal: TimelineModal
var metrics: CombatHudMetrics = null   # the sizes last applied by _fit()
var autoplay: CombatAutoplay = null    # AUTO1 tactics + the Autoplay loop (the timeline row's controls bind to it)

var _panels: Dictionary = {}    # String -> CombatPanel
var _regions: Dictionary = {}   # String -> BoxContainer
var _slots: Dictionary = {}     # String -> MarginContainer (panel + placeholder + edit tag, stacked)
var _windows: Dictionary = {}   # String -> UiWindow (undocked panels)
var _float_layer: Control
var _overlay_layer: UiOverlayLayer
var _edit_bar: PanelContainer
var _layout_option: OptionButton
var _zone_option: OptionButton
var _edit_mode: bool = false
var _auto_timer: Timer
var _autoplay_timer: Timer
var _top_row: Control
var _center_row: Control
var _bottom_panel: PanelContainer
var _columns: Array[Control] = []

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
	margin.add_theme_constant_override("margin_left", CombatHudMetrics.MARGIN_H)
	margin.add_theme_constant_override("margin_right", CombatHudMetrics.MARGIN_H)
	margin.add_theme_constant_override("margin_top", CombatHudMetrics.MARGIN_V)
	margin.add_theme_constant_override("margin_bottom", CombatHudMetrics.MARGIN_V)
	add_child(margin)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", CombatHudMetrics.ROW_SEP)
	margin.add_child(root)

	# Top row: the timeline, with room at the right for the 32 px settings cog.
	var top := _row(root, 4)
	_top_row = top
	top.custom_minimum_size = Vector2(0, CombatHudMetrics.TOP_H)
	_add_region(top, "top", true, true)
	var cog_gap := Control.new()
	cog_gap.custom_minimum_size = Vector2(CORE_COG_PX + 4, 0)
	top.add_child(cog_gap)

	# Centre row: party column | grid | hostile column.
	var center := _row(root, CombatHudMetrics.COL_SEP)
	_center_row = center
	center.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var left_col := _column(center)
	_add_region(left_col, "left_top", false, false)
	_add_region(left_col, "left", false, true)
	_add_region(center, "center", true, true)
	var right_col := _column(center)
	_add_region(right_col, "right_top", false, false)
	_add_region(right_col, "right", false, true)

	_bottom_panel = PanelContainer.new()
	_bottom_panel.custom_minimum_size = Vector2(0, 36)
	_bottom_panel.add_theme_stylebox_override("panel", UiTheme.box(Color(0.07, 0.09, 0.14, 0.98), Color(0.22, 0.32, 0.46, 0.9), 6, 8, 2))
	root.add_child(_bottom_panel)
	var bottom := HBoxContainer.new()
	bottom.add_theme_constant_override("separation", 10)
	_bottom_panel.add_child(bottom)
	_add_region(bottom, "bottom_left", false, true)
	_add_region(bottom, "bottom_center", true, true)

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

# A fixed-width column holding two stacked regions (party alone, or hostiles / forecast).
func _column(parent: Control) -> VBoxContainer:
	var col := VBoxContainer.new()
	col.custom_minimum_size = Vector2(CombatHudMetrics.SIDE_W, 0)
	col.add_theme_constant_override("separation", CombatHudMetrics.COL_GAP)
	col.clip_contents = true
	parent.add_child(col)
	_columns.append(col)
	return col

func _add_region(parent: Control, region: String, expand_h: bool, expand_v: bool) -> void:
	var box := VBoxContainer.new()
	box.name = region
	box.add_theme_constant_override("separation", CombatHudMetrics.COL_GAP)
	if expand_h:
		box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if expand_v:
		box.size_flags_vertical = Control.SIZE_EXPAND_FILL
	if region == "center":
		box.alignment = BoxContainer.ALIGNMENT_CENTER
	parent.add_child(box)
	_regions[region] = box

# ------------------------------------------------------------------------------
# Setup
# ------------------------------------------------------------------------------

# `with_autoplay` false (replay / playback arenas): no CombatAutoplay, no tactic dropdowns, A and Space-pause do nothing.
func setup(p_session: CombatSession, with_autoplay: bool = true) -> void:
	session = p_session
	_overlay_layer.set_session(session)
	var panels: Array[CombatPanel] = [
		CombatRosterPanel.create(true),
		CombatRosterPanel.create(false),
		CombatActionLogPanel.new(),
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
	_wire_hover_card()
	var timeline: CombatTimelinePanel = get_panel("timeline") as CombatTimelinePanel
	if with_autoplay:
		autoplay = CombatAutoplay.new(session)
		if timeline != null:
			timeline.bind_autoplay(autoplay)
		_start_autoplay_timer()
	elif timeline != null:
		timeline.disable_autoplay()

	log_modal.cell_clicked.connect(func(e: ActionLogEntry) -> void: session.select_log_entry(e))
	timeline_modal.action_lookup_fn = session.find_recorded_action
	timeline_modal.token_texture_fn = session.encounter.get_token_texture
	timeline_modal.round_nav_requested.connect(func(delta: int) -> void: session.navigate_round(delta))
	timeline_modal.block_selected.connect(func(b: int) -> void: session.select_block(b))
	timeline_modal.combatant_selected.connect(func(c: Combatant) -> void: session.select_combatant(c))
	session.changed.connect(_on_session_changed)
	apply_layout(DEFAULT_LAYOUT)
	_sync_zone_option()
	resized.connect(_fit)
	layout_changed.connect(_fit)
	session.mounted.connect(_fit)
	_fit()

# Drives CombatAutoplay.tick() (draft, confirm, advance); stops itself when the battle is over.
func _start_autoplay_timer() -> void:
	_autoplay_timer = Timer.new()
	_autoplay_timer.wait_time = AUTOPLAY_TICK
	_autoplay_timer.timeout.connect(func() -> void:
		if session == null or session.combat_over:
			_autoplay_timer.stop()
		elif autoplay != null and not keys_blocked:
			autoplay.tick()
	)
	add_child(_autoplay_timer)
	_autoplay_timer.start()
	session.round_started.connect(func(_round_num: int) -> void:   # a reset battle starts ticking again
		if _autoplay_timer.is_stopped() and _autoplay_timer.is_inside_tree():
			_autoplay_timer.start()
	)

# A battle that belongs to a save remembers the tactics and the Autoplay switch in its GameState (AUTO1 rule 8).
func bind_game_state(store: GameState) -> void:
	if autoplay == null or store == null:
		return
	autoplay.store = store
	autoplay.load_settings(store.autoplay)

# Roster cards drive the grid's hover card: focus opens it at once, a click pins it.
func _wire_hover_card() -> void:
	var grid: CombatGridPanel = get_panel("grid") as CombatGridPanel
	for id: String in ["party", "enemies"]:
		var roster: CombatRosterPanel = get_panel(id) as CombatRosterPanel
		if roster == null or grid == null:
			continue
		roster.unit_focused.connect(grid.open_card)
		roster.unit_blurred.connect(grid.focus_lost)
		roster.unit_clicked.connect(grid.pin_card)

# Sizes the whole screen from CombatHudMetrics: the grid's tile, the columns' fixed panel heights, the strips.
func _fit() -> void:
	if session == null or _panels.is_empty():
		return
	var view: Vector2 = size if size.x > 1.0 and size.y > 1.0 else Vector2(1280, 720)
	metrics = CombatHudMetrics.compute(view, UiTheme.text_scale(), session.allies.size(), maxi(session.enemies.size(), CombatHudMetrics.HOSTILE_SLOTS), session.encounter.party_slots())
	_top_row.custom_minimum_size = Vector2(0, CombatHudMetrics.TOP_H)
	_bottom_panel.custom_minimum_size = Vector2(0, metrics.bottom_h)
	_set_slot_height("party", metrics.party_panel_h)
	_set_slot_height("enemies", metrics.hostile_panel_h)
	_set_slot_height("forecast", metrics.forecast_h)
	for id: String in ["party", "enemies"]:
		var roster: CombatRosterPanel = get_panel(id) as CombatRosterPanel
		if roster != null:
			roster.apply_metrics(metrics)
	var grid: CombatGridPanel = get_panel("grid") as CombatGridPanel
	if grid != null:
		grid.set_tile_px(metrics.tile)
		var board: int = metrics.tile * CombatGridView.GRID_W + CombatGridView.FRAME_PX
		_set_slot_height("grid", metrics.tile * CombatGridView.GRID_H + CombatGridView.FRAME_PX)
		_slot("grid").custom_minimum_size.x = board

func _set_slot_height(panel_id: String, h: int) -> void:
	var slot: MarginContainer = _slot(panel_id)
	if slot != null:
		slot.custom_minimum_size = Vector2(0, h)
		slot.size_flags_vertical = Control.SIZE_SHRINK_BEGIN

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
	for region_id: String in REGIONS:   # an empty region takes no room (its column separator would eat height)
		var box: BoxContainer = _region(region_id)
		box.visible = box.get_child_count() > 0
	if _layout_option != null:
		for i: int in range(_layout_option.item_count):
			if _layout_option.get_item_text(i) == layout_name:
				_layout_option.select(i)
	layout_changed.emit()

func _apply_region_flags(slot: Control, region: String) -> void:
	match region:
		"top", "bottom_center":
			slot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			slot.size_flags_vertical = Control.SIZE_EXPAND_FILL
		"center":
			slot.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
		"left", "right":
			slot.size_flags_horizontal = Control.SIZE_FILL
			slot.size_flags_vertical = Control.SIZE_EXPAND_FILL
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
	corner.name = "SettingsCog"
	corner.text = "⚙"
	corner.tooltip_text = "Layout edit mode [F9]: undock panels, switch layout presets"
	corner.focus_mode = Control.FOCUS_NONE
	corner.add_theme_font_size_override("font_size", UiTheme.hud_font(18))
	corner.add_theme_stylebox_override("normal", UiTheme.box(Color(0.10, 0.14, 0.20, 0.95), Color(0.38, 0.5, 0.68, 0.9), 4, 2, 2))
	corner.add_theme_stylebox_override("hover", UiTheme.box(Color(0.16, 0.22, 0.32, 0.95), Color(0.55, 0.75, 1.0, 1.0), 4, 2, 2))
	corner.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT)
	corner.offset_left = -(CombatHudMetrics.MARGIN_H + CORE_COG_PX) + 12
	corner.offset_right = corner.offset_left + CORE_COG_PX
	corner.offset_top = CombatHudMetrics.MARGIN_V
	corner.offset_bottom = CombatHudMetrics.MARGIN_V + CORE_COG_PX
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
	row.add_child(UiTheme.label("Zone", UiTheme.FONT_SM, UiTheme.COL_WARN))
	_zone_option = OptionButton.new()
	_zone_option.focus_mode = Control.FOCUS_NONE
	_zone_option.tooltip_text = "Rebuild the battle on another terrain preset (restarts the encounter)"
	_zone_option.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	for zone_id: String in CombatZones.zone_ids():
		_zone_option.add_item(CombatZones.zone_name(zone_id))
		_zone_option.set_item_metadata(_zone_option.item_count - 1, zone_id)
	_zone_option.item_selected.connect(_on_zone_selected)
	row.add_child(_zone_option)
	row.add_child(UiTheme.button("Dock All", "Return every floating panel to its dock", dock_all, UiTheme.FONT_SM))
	row.add_child(UiTheme.button("✔ Done", "Leave layout edit mode", func() -> void: set_edit_mode(false), UiTheme.FONT_SM))

func _on_zone_selected(index: int) -> void:
	var meta: Variant = _zone_option.get_item_metadata(index)
	if meta is String:
		var zone_id: String = meta
		select_zone(zone_id)

# Switches the encounter's terrain preset and restarts the battle on it.
func select_zone(zone_id: String) -> void:
	if session == null or not CombatZones.ZONES.has(zone_id):
		return
	session.encounter.zone_id = zone_id
	session.reset_battle()
	_sync_zone_option()

func _sync_zone_option() -> void:
	if _zone_option == null or session == null:
		return
	for i: int in range(_zone_option.item_count):
		if _zone_option.get_item_metadata(i) == session.encounter.zone_id:
			_zone_option.select(i)

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

func close_modals() -> bool:
	if log_modal.visible:
		log_modal.close_modal()
		return true
	if timeline_modal.visible:
		timeline_modal.close_modal()
		return true
	return false

# Esc closes the unit hover card (true when one was open).
func close_hover_card() -> bool:
	var grid: CombatGridPanel = get_panel("grid") as CombatGridPanel
	if grid == null or not grid.card_is_open():
		return false
	grid.close_card()
	return true

func _refresh_timeline_modal() -> void:
	timeline_modal.set_total_blocks(session.total_blocks)
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

# ------------------------------------------------------------------------------
# Keyboard shortcuts
# ------------------------------------------------------------------------------

func _unhandled_input(event: InputEvent) -> void:
	var key := event as InputEventKey
	if key == null or not key.pressed or key.echo or session == null or keys_blocked:
		return
	var handled: bool = true
	match key.keycode:
		KEY_ESCAPE:
			handled = close_modals() or close_hover_card()
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
		KEY_BACKSPACE:
			if session.is_planning():
				session.plan_undo_last()
			else:
				handled = false
		KEY_ENTER, KEY_KP_ENTER:
			if session.is_planning():
				session.confirm_plan()
			else:
				handled = false
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
		KEY_A:
			if autoplay != null:
				autoplay.toggle()   # A: Autoplay on / off (off = the same pause as Space)
			else:
				handled = false
		KEY_SPACE:
			if autoplay != null and autoplay.is_on():
				autoplay.request_pause()   # Space pauses autoplay: this round finishes, the next stops in planning
			else:
				session.toggle_auto_play()
		_:
			handled = false
	if handled:
		get_viewport().set_input_as_handled()
