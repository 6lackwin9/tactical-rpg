class_name CombatLogModal
extends PanelContainer

# ==============================================================================
# Tactical RPG - Expanded Action Stream Log Modal
# Full-screen modal overlay displaying chronological, categorized action cells
# ==============================================================================

signal cell_clicked(entry: ActionLogEntry)
signal modal_closed()

var entries_vbox: VBoxContainer
var filter_buttons: Dictionary = {} # String -> Button
var current_filter: String = "ALL"
var all_entries: Array[ActionLogEntry] = []

func _init() -> void:
	name = "CombatLogModal"
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	_build_ui()

func _build_ui() -> void:
	var backdrop_style := StyleBoxFlat.new()
	backdrop_style.bg_color = Color(0.02, 0.04, 0.07, 0.9)
	add_theme_stylebox_override("panel", backdrop_style)

	var center_margin := MarginContainer.new()
	center_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	center_margin.add_theme_constant_override("margin_left", 180)
	center_margin.add_theme_constant_override("margin_right", 180)
	center_margin.add_theme_constant_override("margin_top", 70)
	center_margin.add_theme_constant_override("margin_bottom", 70)
	add_child(center_margin)

	var modal_box := PanelContainer.new()
	var box_style := StyleBoxFlat.new()
	box_style.bg_color = Color(0.07, 0.09, 0.14, 0.98)
	box_style.border_color = Color(0.28, 0.46, 0.75, 0.9)
	box_style.set_border_width_all(2)
	box_style.set_corner_radius_all(6)
	box_style.content_margin_left = 12
	box_style.content_margin_right = 12
	box_style.content_margin_top = 10
	box_style.content_margin_bottom = 10
	modal_box.add_theme_stylebox_override("panel", box_style)
	center_margin.add_child(modal_box)

	var modal_vbox := VBoxContainer.new()
	modal_vbox.add_theme_constant_override("separation", 6)
	modal_box.add_child(modal_vbox)

	# 1. Top header bar
	var hdr := HBoxContainer.new()
	modal_vbox.add_child(hdr)

	var title := Label.new()
	title.text = "📜 EXPANDED ACTION STREAM LOG"
	title.add_theme_font_size_override("font_size", 12)
	title.add_theme_color_override("font_color", Color(0.96, 0.85, 0.35))
	hdr.add_child(title)

	var sub := Label.new()
	sub.text = "Chronological Action Cells (R<X>-B<Y> Reference Format)"
	sub.add_theme_font_size_override("font_size", 8)
	sub.add_theme_color_override("font_color", Color(0.55, 0.65, 0.78))
	hdr.add_child(sub)

	hdr.add_spacer(false)

	var close_btn := Button.new()
	close_btn.text = "✕ CLOSE (ESC / L)"
	close_btn.custom_minimum_size = Vector2(95, 22)
	close_btn.add_theme_font_size_override("font_size", 9)
	var cls_style := StyleBoxFlat.new()
	cls_style.bg_color = Color(0.35, 0.12, 0.15, 0.9)
	cls_style.border_color = Color(0.85, 0.35, 0.35, 1.0)
	cls_style.set_border_width_all(1)
	cls_style.set_corner_radius_all(3)
	close_btn.add_theme_stylebox_override("normal", cls_style)
	close_btn.pressed.connect(func() -> void: close_modal())
	hdr.add_child(close_btn)

	# 2. Category filter bar
	var filter_bar := HBoxContainer.new()
	filter_bar.add_theme_constant_override("separation", 4)
	modal_vbox.add_child(filter_bar)

	var filter_hdr := Label.new()
	filter_hdr.text = "Filter Stream:"
	filter_hdr.add_theme_font_size_override("font_size", 8)
	filter_hdr.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
	filter_bar.add_child(filter_hdr)

	var categories: Array[String] = ["ALL", "ATTACK", "DEFEND", "BUFF", "MOVE", "TARGET", "EVENT"]
	for cat in categories:
		var btn := Button.new()
		btn.text = cat
		btn.custom_minimum_size = Vector2(50, 18)
		btn.add_theme_font_size_override("font_size", 8)
		var current_cat := cat
		btn.pressed.connect(func() -> void:
			current_filter = current_cat
			refresh_content()
		)
		filter_bar.add_child(btn)
		filter_buttons[cat] = btn

	var hsep := HSeparator.new()
	modal_vbox.add_child(hsep)

	# 3. Scrollable entries stream
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	modal_vbox.add_child(scroll)

	entries_vbox = VBoxContainer.new()
	entries_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	entries_vbox.add_theme_constant_override("separation", 4)
	scroll.add_child(entries_vbox)

func open_modal() -> void:
	visible = true
	move_to_front()
	refresh_content()

func close_modal() -> void:
	visible = false
	modal_closed.emit()

func toggle_modal() -> void:
	if visible:
		close_modal()
	else:
		open_modal()

func is_modal_open() -> bool:
	return visible

func set_entries(p_entries: Array[ActionLogEntry]) -> void:
	all_entries = p_entries
	if visible:
		refresh_content()

func refresh_content() -> void:
	if not entries_vbox:
		return
	for child in entries_vbox.get_children():
		child.queue_free()

	# Highlight active filter button
	for cat: String in filter_buttons:
		var btn: Button = filter_buttons[cat]
		if cat == current_filter:
			btn.add_theme_color_override("font_color", Color(0.95, 0.85, 0.35))
		else:
			btn.remove_theme_color_override("font_color")

	if all_entries.is_empty():
		var empty_lbl := Label.new()
		empty_lbl.text = "No action cells resolved yet. Run combat blocks to populate stream."
		empty_lbl.add_theme_font_size_override("font_size", 10)
		empty_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		entries_vbox.add_child(empty_lbl)
		return

	var last_round: int = -1
	var last_block: int = -1

	for entry in all_entries:
		if current_filter != "ALL" and entry.category != current_filter:
			continue

		# Round Divider
		if entry.round_num != last_round:
			last_round = entry.round_num
			last_block = -1
			var rd_panel := PanelContainer.new()
			var rd_style := StyleBoxFlat.new()
			rd_style.bg_color = Color(0.12, 0.16, 0.24, 0.95)
			rd_style.border_color = Color(0.35, 0.55, 0.85, 0.9)
			rd_style.set_border_width_all(1)
			rd_style.set_corner_radius_all(3)
			rd_style.content_margin_left = 8
			rd_style.content_margin_right = 8
			rd_style.content_margin_top = 2
			rd_style.content_margin_bottom = 2
			rd_panel.add_theme_stylebox_override("panel", rd_style)

			var rd_lbl := Label.new()
			rd_lbl.text = "━━━ ROUND %d RESOLUTION STREAM ━━━" % entry.round_num
			rd_lbl.add_theme_font_size_override("font_size", 9)
			rd_lbl.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
			rd_panel.add_child(rd_lbl)
			entries_vbox.add_child(rd_panel)

		# Block Divider
		if entry.block_num != last_block:
			last_block = entry.block_num
			var blk_lbl := Label.new()
			blk_lbl.text = "── Block %02d Actions ──" % entry.block_num
			blk_lbl.add_theme_font_size_override("font_size", 8)
			blk_lbl.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
			entries_vbox.add_child(blk_lbl)

		var cur_entry := entry
		var cell := format_action_cell(cur_entry, true, func(e: ActionLogEntry) -> void:
			cell_clicked.emit(e)
		)
		entries_vbox.add_child(cell)

static func format_action_cell(entry: ActionLogEntry, is_modal: bool = false, on_cell_clicked: Callable = Callable()) -> PanelContainer:
	var cell := PanelContainer.new()
	cell.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cell.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND if is_modal else Control.CURSOR_ARROW

	var style := StyleBoxFlat.new()
	var cat_color: Color
	match entry.category:
		"ATTACK": cat_color = Color(0.92, 0.32, 0.32)
		"DEFEND": cat_color = Color(0.35, 0.68, 1.0)
		"BUFF": cat_color = Color(0.98, 0.78, 0.25)
		"MOVE": cat_color = Color(0.3, 0.8, 0.95)
		"TARGET": cat_color = Color(0.3, 0.85, 0.55)
		"EVENT": cat_color = Color(0.75, 0.5, 0.98)
		_: cat_color = Color(0.7, 0.75, 0.85)

	style.bg_color = Color(cat_color.r * 0.12, cat_color.g * 0.12, cat_color.b * 0.15, 0.95)
	style.border_color = cat_color.lerp(Color.WHITE, 0.15) if is_modal else cat_color
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 6
	style.content_margin_right = 6
	style.content_margin_top = 2
	style.content_margin_bottom = 2
	cell.add_theme_stylebox_override("panel", style)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 1)
	cell.add_child(vbox)

	# Row 1: Badges & Tags
	var row1 := HBoxContainer.new()
	row1.add_theme_constant_override("separation", 5)
	vbox.add_child(row1)

	# Reference Tag Badge [R1-B04]
	var ref_lbl := Label.new()
	ref_lbl.text = "[%s]" % entry.ref_tag
	ref_lbl.add_theme_font_size_override("font_size", 9)
	ref_lbl.add_theme_color_override("font_color", Color(0.95, 0.85, 0.35))
	row1.add_child(ref_lbl)

	# Category + Subtype Badge
	var type_lbl := Label.new()
	type_lbl.text = "• %s" % entry.category
	if not entry.skill_subtype.is_empty():
		type_lbl.text += " (%s)" % entry.skill_subtype
	type_lbl.add_theme_font_size_override("font_size", 8)
	type_lbl.add_theme_color_override("font_color", cat_color)
	row1.add_child(type_lbl)

	row1.add_spacer(false)

	# Outcome Badge
	if not entry.outcome_tag.is_empty():
		var out_lbl := Label.new()
		out_lbl.text = "[%s]" % entry.outcome_tag
		out_lbl.add_theme_font_size_override("font_size", 8)
		if entry.outcome_tag == "DEFEATED":
			out_lbl.add_theme_color_override("font_color", Color(1.0, 0.25, 0.25))
		elif "MITIGAT" in entry.outcome_tag:
			out_lbl.add_theme_color_override("font_color", Color(0.4, 0.85, 1.0))
		elif "BUFF" in entry.outcome_tag or "+" in entry.outcome_tag:
			out_lbl.add_theme_color_override("font_color", Color(0.98, 0.82, 0.3))
		elif entry.outcome_tag == "WITHDRAWN":
			out_lbl.add_theme_color_override("font_color", Color(0.35, 0.95, 0.55))
		else:
			out_lbl.add_theme_color_override("font_color", Color(0.9, 0.9, 0.95))
		row1.add_child(out_lbl)

	# Row 2: Narrative & Target Details
	var row2 := HBoxContainer.new()
	row2.add_theme_constant_override("separation", 4)
	vbox.add_child(row2)

	# Actor
	var actor_lbl := Label.new()
	actor_lbl.text = entry.actor_name
	actor_lbl.add_theme_font_size_override("font_size", 9)
	var actor_col := Color(0.4, 0.75, 1.0) if entry.actor_is_ally else Color(1.0, 0.45, 0.45)
	actor_lbl.add_theme_color_override("font_color", actor_col)
	row2.add_child(actor_lbl)

	# Verb / Skill
	var skill_lbl := Label.new()
	var icon_sym := "⚔"
	match entry.category:
		"MOVE": icon_sym = "➜"
		"DEFEND": icon_sym = "🛡"
		"BUFF": icon_sym = "✦"
		"TARGET": icon_sym = "🎯"
		"EVENT": icon_sym = "⚡"
	skill_lbl.text = "%s %s" % [icon_sym, entry.skill_name]
	skill_lbl.add_theme_font_size_override("font_size", 8)
	skill_lbl.add_theme_color_override("font_color", Color(0.85, 0.85, 0.9))
	row2.add_child(skill_lbl)

	# Target (if any)
	if not entry.target_name.is_empty():
		var arrow_lbl := Label.new()
		arrow_lbl.text = "➔"
		arrow_lbl.add_theme_font_size_override("font_size", 8)
		arrow_lbl.add_theme_color_override("font_color", Color(0.6, 0.65, 0.75))
		row2.add_child(arrow_lbl)

		var target_lbl := Label.new()
		target_lbl.text = entry.target_name
		target_lbl.add_theme_font_size_override("font_size", 9)
		var target_col := Color(0.4, 0.75, 1.0) if entry.target_is_ally else Color(1.0, 0.45, 0.45)
		target_lbl.add_theme_color_override("font_color", target_col)
		row2.add_child(target_lbl)

	row2.add_spacer(false)

	# Status / Detail Text
	var detail_msg: String = entry.status_text if not entry.status_text.is_empty() else entry.detail_text
	if not detail_msg.is_empty():
		var stat_lbl := Label.new()
		stat_lbl.text = detail_msg
		stat_lbl.add_theme_font_size_override("font_size", 8)
		stat_lbl.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		row2.add_child(stat_lbl)

	if is_modal and on_cell_clicked.is_valid():
		cell.gui_input.connect(func(ev: InputEvent) -> void:
			var mb := ev as InputEventMouseButton
			if mb != null and mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT:
				on_cell_clicked.call(entry)
		)

	return cell
