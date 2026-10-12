class_name DojoLogPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 6: Test Log
# Renders dojo events with the exact same action-cell UI as the combat log
# (CombatLogModal.format_action_cell). Click a cell to see its full math
# breakdown in the detail pane below.
# ==============================================================================

const FILTERS: Array[String] = ["ALL", "ATTACK", "DEFEND", "BUFF", "EVENT"]

var _filter: String = "ALL"
var _filter_btns: Dictionary = {} # String -> Button
var _scroll: ScrollContainer
var _entries_vbox: VBoxContainer
var _detail: RichTextLabel
var _pending_scroll: bool = false

func _init() -> void:
	super._init()
	panel_id = "log"
	panel_title = "Test Log"
	panel_short_title = "Log"
	panel_icon = "📜"
	default_min_size = Vector2(340, 180)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 4)
	add_child(vbox)

	var bar := HFlowContainer.new()
	bar.add_theme_constant_override("h_separation", 3)
	bar.add_theme_constant_override("v_separation", 3)
	vbox.add_child(bar)
	for cat: String in FILTERS:
		var btn: Button = UiTheme.button(cat, "Show %s entries" % cat, func() -> void: _set_filter(cat), 9)
		btn.toggle_mode = true
		bar.add_child(btn)
		_filter_btns[cat] = btn
	var clear_btn: Button = UiTheme.button("🗑 Clear", "Clear the test log", func() -> void: session.clear_log(), 9)
	bar.add_child(clear_btn)

	var split := VSplitContainer.new()
	split.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vbox.add_child(split)

	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.custom_minimum_size = Vector2(0, 90)
	split.add_child(_scroll)
	_scroll.get_v_scroll_bar().changed.connect(_on_scroll_range_changed)

	_entries_vbox = VBoxContainer.new()
	_entries_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_entries_vbox.add_theme_constant_override("separation", 3)
	_scroll.add_child(_entries_vbox)

	var detail_frame := PanelContainer.new()
	detail_frame.custom_minimum_size = Vector2(0, 90)
	detail_frame.size_flags_vertical = Control.SIZE_EXPAND_FILL
	detail_frame.size_flags_stretch_ratio = 0.7
	detail_frame.add_theme_stylebox_override("panel", UiTheme.box(Color(0.05, 0.07, 0.10, 0.95), Color(0.18, 0.25, 0.35, 0.8), 4, 8, 6))
	split.add_child(detail_frame)

	_detail = RichTextLabel.new()
	_detail.bbcode_enabled = true
	_detail.scroll_following = false
	_detail.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	_detail.add_theme_font_size_override("bold_font_size", UiTheme.FONT_SM)
	detail_frame.add_child(_detail)

	_update_filter_buttons()
	_show_detail(null)

func _on_bound() -> void:
	session.entry_logged.connect(_on_entry_logged)
	session.log_cleared.connect(_rebuild)
	_rebuild()

func refresh() -> void:
	pass # Event-driven: the log only changes through entry_logged / log_cleared.

# Number of log cells currently rendered (excludes the empty-state placeholder).
func displayed_entry_count() -> int:
	return _entries_vbox.get_child_count() if not session.log_entries.is_empty() else 0

func _set_filter(category: String) -> void:
	_filter = category
	_update_filter_buttons()
	_rebuild()

func _update_filter_buttons() -> void:
	for cat: String in _filter_btns:
		var btn_raw: Variant = _filter_btns[cat]
		if btn_raw is Button:
			var btn: Button = btn_raw
			btn.set_pressed_no_signal(cat == _filter)

func _passes_filter(entry: ActionLogEntry) -> bool:
	return _filter == "ALL" or entry.category == _filter

func _rebuild() -> void:
	UiTheme.clear_children(_entries_vbox)
	if session.log_entries.is_empty():
		_entries_vbox.add_child(UiTheme.label("No test actions yet. Use the Test Console or change gear.", UiTheme.FONT_SM, UiTheme.COL_MUTED))
		_show_detail(null)
		return
	var last: DojoLogEntry = null
	for entry: DojoLogEntry in session.log_entries:
		if _passes_filter(entry):
			_add_cell(entry)
			last = entry
	_show_detail(last)
	_pending_scroll = true

func _on_entry_logged(entry: DojoLogEntry) -> void:
	if session.log_entries.size() == 1:
		UiTheme.clear_children(_entries_vbox)
	if not _passes_filter(entry):
		return
	_add_cell(entry)
	_show_detail(entry)
	_pending_scroll = true

func _add_cell(entry: DojoLogEntry) -> void:
	var cell: PanelContainer = CombatLogModal.format_action_cell(entry, true, _on_cell_clicked)
	_entries_vbox.add_child(cell)

func _on_cell_clicked(entry: ActionLogEntry) -> void:
	if entry is DojoLogEntry:
		var dojo_entry: DojoLogEntry = entry
		_show_detail(dojo_entry)

func _show_detail(entry: DojoLogEntry) -> void:
	_detail.clear()
	if entry == null:
		_detail.append_text("[color=#64748b]Select a log entry to inspect its step-by-step calculation.[/color]")
	elif entry.breakdown_bbcode.is_empty():
		_detail.append_text("[color=#94a3b8][b]%s[/b]  %s[/color]" % [entry.skill_name, entry.detail_text])
	else:
		_detail.append_text(entry.breakdown_bbcode)

func _on_scroll_range_changed() -> void:
	if _pending_scroll:
		_pending_scroll = false
		_scroll.scroll_vertical = int(_scroll.get_v_scroll_bar().max_value)
